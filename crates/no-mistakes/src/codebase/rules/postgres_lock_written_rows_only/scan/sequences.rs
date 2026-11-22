use super::*;
use crate::codebase::postgres::{EmbeddedSqlOptions, SqlStatementFileFacts};

mod share;

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
        share::scan(output, file, source, calls, grouped, unresolved);
    }
    if options.track_redundant_prelocks {
        redundant_prelocks(output, file, source, calls, grouped, unresolved);
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

fn transaction_scope(call: &EmbeddedSqlCall) -> Option<Transaction> {
    let id = if call.executor_transaction_known {
        call.transaction_id
    } else {
        call.caller_transaction_id
    }?;
    (call.execution_range.0 < call.execution_range.1).then_some((id, call.execution_scope))
}

fn ordered_transaction(call: &EmbeddedSqlCall, unresolved: &BTreeSet<u32>) -> Option<Transaction> {
    let scope = transaction_scope(call)?;
    (call.lock_helper.is_none()
        && !unresolved.contains(&scope.0)
        && call.execution_unconditional
        && call.execution_awaited)
        .then_some(scope)
}
