use super::*;
use crate::codebase::postgres::{EmbeddedSqlOptions, SqlStatementFileFacts};

type Transaction = (u32, u32);

pub(super) fn scan(
    output: &mut Output<'_>,
    (file, source): (&str, &str),
    calls: &[EmbeddedSqlCall],
    grouped: &BTreeMap<usize, Vec<&SqlStatementFileFacts>>,
    unresolved: &BTreeSet<u32>,
    options: &EmbeddedSqlOptions,
) {
    if options.track_share_lock_writes {
        share_locks(output, file, source, calls, grouped, unresolved);
    }
    if options.track_redundant_prelocks {
        redundant_prelocks(output, file, source, calls, grouped, unresolved);
    }
}

fn share_locks(
    output: &mut Output<'_>,
    file: &str,
    source: &str,
    calls: &[EmbeddedSqlCall],
    grouped: &BTreeMap<usize, Vec<&SqlStatementFileFacts>>,
    unresolved: &BTreeSet<u32>,
) {
    let mut ordered = Vec::new();
    let candidates = complete_calls(calls, grouped)
        .filter(|(_, facts)| !facts.lock_writes.share_statements.is_empty());
    for (call, facts) in candidates {
        if let Some(transaction) = ordered_transaction(call, unresolved) {
            ordered.push((transaction, call, facts));
        } else if !known_autocommit(call)
            && facts
                .lock_writes
                .share_statements
                .iter()
                .any(|statement| !statement.locks.is_empty())
        {
            output.unresolved(
                &Site { file, source, call },
                "share lock transaction or statement order is unresolved; use a proven transaction executor and await the lock and write",
            );
        }
    }
    ordered.sort_unstable_by_key(|(transaction, call, _)| (*transaction, call.execution_range.1));
    let mut current = None;
    let mut written = BTreeSet::new();
    for (transaction, call, facts) in ordered.into_iter().rev() {
        if current != Some(transaction) {
            current = Some(transaction);
            written.clear();
        }
        let access = &facts.lock_writes;
        let mut reported = BTreeSet::new();
        for statement in access.share_statements.iter().rev() {
            let site = Site { file, source, call };
            for relation in &statement.locks {
                match relation.as_deref() {
                    Some(relation) if written.contains(relation) && reported.insert(Some(relation)) => {
                        output.emit(
                            &site,
                            super::super::SHARE_LOCK_THEN_WRITE,
                            Some(relation),
                            None,
                            "share lock precedes an update or delete of this relation in the same transaction; take a write-strength lock up front or use a conditional write",
                        );
                    }
                    None if reported.insert(None) => output.unresolved(
                        &site,
                        "cannot prove shared lock execution on a base relation; use a top-level lock and an explicit relation or FOR SHARE OF alias",
                    ),
                    _ => {}
                }
            }
            written.extend(
                access.writes[statement.writes.clone()]
                    .iter()
                    .filter_map(|row| row.relation.as_deref()),
            );
        }
    }
}

fn redundant_prelocks(
    output: &mut Output<'_>,
    file: &str,
    source: &str,
    calls: &[EmbeddedSqlCall],
    grouped: &BTreeMap<usize, Vec<&SqlStatementFileFacts>>,
    unresolved: &BTreeSet<u32>,
) {
    let candidates = complete_calls(calls, grouped);
    let writes: BTreeMap<_, _> = candidates
        .clone()
        .filter(|(_, facts)| !facts.lock_writes.conditional_write_keys.is_empty())
        .map(|(call, facts)| {
            (
                call.execution_range.0,
                (call, &facts.lock_writes.conditional_write_keys),
            )
        })
        .collect();
    for (call, facts) in candidates {
        let Some(next) = call
            .next_discarded_query
            .filter(|_| facts.lock_writes.prelock_attempt)
        else {
            continue;
        };
        let Some((write_call, keys)) = writes.get(&next) else {
            continue;
        };
        if known_autocommit(call) || known_autocommit(write_call) {
            continue;
        }
        let site = Site { file, source, call };
        let lock_tx = ordered_transaction(call, unresolved);
        let write_tx = ordered_transaction(write_call, unresolved);
        if matches!((lock_tx, write_tx), (Some(left), Some(right)) if left != right) {
            continue;
        }
        let Some(lock) = &facts.lock_writes.prelock else {
            if facts.lock_writes.locks.iter().any(|lock| {
                lock.relation.is_none() || keys.iter().any(|write| write.relation == lock.relation)
            }) {
                output.unresolved(&site, "cannot prove the discarded lock reads only a static key and has no other effects");
            }
            continue;
        };
        let (mut matches, mut unknown) = (false, false);
        for write in keys.iter().filter(|write| write.relation == lock.relation) {
            if lock.key.as_ref().map(|(column, _)| column)
                != write.key.as_ref().map(|(column, _)| column)
            {
                continue;
            }
            let same = rows::compare_keys(lock, call, write, write_call);
            matches |= same == Some(true);
            unknown |= same.is_none();
        }
        if matches
            && lock_tx.is_some()
            && write_tx.is_some()
            && call.stable_prelock_bindings
            && write_call.stable_prelock_bindings
        {
            output.emit(
                &site,
                super::super::REDUNDANT_PRELOCK,
                lock.relation.as_deref(),
                lock.key.as_ref().map(|(_, value)| value.as_str()),
                "the following conditional write decides whether to change the locked row; remove the unused pre-lock",
            );
        } else if matches || unknown {
            output.unresolved(
                &site,
                "cannot prove the pre-lock and conditional write share a transaction and immutable row key",
            );
        }
    }
}

fn complete_calls<'a>(
    calls: &'a [EmbeddedSqlCall],
    grouped: &'a BTreeMap<usize, Vec<&'a SqlStatementFileFacts>>,
) -> impl Iterator<Item = (&'a EmbeddedSqlCall, &'a SqlStatementFileFacts)> + Clone {
    calls.iter().enumerate().filter_map(|(index, call)| {
        let [facts] = grouped.get(&index)?.as_slice() else {
            return None;
        };
        (!call.helper_definition && !call.is_unanalyzable() && !facts.parse_failed)
            .then_some((call, *facts))
    })
}

fn known_autocommit(call: &EmbeddedSqlCall) -> bool {
    call.executor_transaction_known && call.transaction_id.is_none()
}

fn ordered_transaction(call: &EmbeddedSqlCall, unresolved: &BTreeSet<u32>) -> Option<Transaction> {
    let id = if call.executor_transaction_known {
        call.transaction_id
    } else {
        call.caller_transaction_id
    }?;
    (call.lock_helper.is_none()
        && !unresolved.contains(&id)
        && call.execution_unconditional
        && call.execution_awaited
        && call.execution_range.0 < call.execution_range.1)
        .then_some((id, call.execution_scope))
}
