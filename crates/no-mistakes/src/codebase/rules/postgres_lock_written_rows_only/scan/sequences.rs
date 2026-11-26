use super::*;
use crate::codebase::postgres::{EmbeddedSqlOptions, SqlStatementFileFacts};

mod later_access;
mod relations;
mod share;
mod unique;
mod variants;

type Transaction = (u32, u32);

pub(super) fn scan(
    output: &mut Output<'_>,
    (file, source): (&str, &str),
    calls: &[EmbeddedSqlCall],
    grouped: &BTreeMap<usize, Vec<&SqlStatementFileFacts>>,
    unresolved: &BTreeSet<u32>,
    options: &EmbeddedSqlOptions,
    catalog: unique::CatalogContext<'_, '_>,
) {
    if options.track_share_lock_writes {
        share::scan(output, (file, source), calls, grouped, unresolved, catalog);
    }
    if options.track_redundant_prelocks {
        variants::scan_redundant(output, file, source, calls, grouped);
        redundant_prelocks(output, file, source, calls, grouped, unresolved, catalog);
    }
}

fn redundant_prelocks(
    output: &mut Output<'_>,
    file: &str,
    source: &str,
    calls: &[EmbeddedSqlCall],
    grouped: &BTreeMap<usize, Vec<&SqlStatementFileFacts>>,
    unresolved: &BTreeSet<u32>,
    catalog: unique::CatalogContext<'_, '_>,
) {
    let candidates = complete_calls(calls, grouped);
    let later_access = std::cell::OnceCell::new();
    let writes: BTreeMap<_, _> = candidates
        .clone()
        .filter(|(_, facts)| {
            !facts.lock_writes.conditional_write_keys.is_empty()
                || facts.lock_writes.multi_row_upsert_keys.is_some()
        })
        .map(|(call, facts)| (call.execution_range.0, (call, &facts.lock_writes)))
        .collect();
    for (call, facts) in candidates {
        let Some(next) = call
            .next_discarded_query
            .filter(|_| facts.lock_writes.prelock_attempt)
        else {
            continue;
        };
        let Some((write_call, write_facts)) = writes.get(&next) else {
            continue;
        };
        let keys = &write_facts.conditional_write_keys;
        if known_autocommit(call) || known_autocommit(write_call) {
            continue;
        }
        let site = Site { file, source, call };
        let lock_tx = ordered_transaction(call, unresolved);
        let write_tx = ordered_transaction(write_call, unresolved);
        if matches!((lock_tx, write_tx), (Some(left), Some(right)) if left != right) {
            continue;
        }
        if let Some(keys) = &write_facts.multi_row_upsert_keys {
            let same_transaction = lock_tx.is_some() && lock_tx == write_tx;
            if same_transaction && facts.lock_writes.prelock.as_ref().is_some_and(|(lock, _)| {
                rows::stable_prelock_key(lock, call) && keys.iter().any(|write| {
                    relations::same(lock.relation.as_deref(), write.relation.as_deref()) == Some(true)
                        && rows::stable_prelock_key(write, write_call)
                        && rows::compare_keys(lock, call, write, write_call) == Some(true)
                })
            }) {
                output.unresolved(
                    &site,
                    "the following multi-row upsert can affect the locked key; cannot prove the pre-lock is redundant",
                );
            }
            continue;
        }
        let Some((lock, only)) = &facts.lock_writes.prelock else {
            if facts.lock_writes.locks.iter().any(|lock| {
                keys.iter().any(|write| {
                    relations::same(lock.relation.as_deref(), write.relation.as_deref())
                        != Some(false)
                })
            }) {
                output.unresolved(&site, "cannot prove the discarded lock reads only a static key and has no other effects");
            }
            continue;
        };
        let (mut matches, mut unknown_key, mut unknown_relation) = (false, false, false);
        for write in keys {
            let same_relation =
                relations::same(lock.relation.as_deref(), write.relation.as_deref());
            if same_relation == Some(false) {
                continue;
            }
            if lock.key.as_ref().map(|(column, _)| column)
                != write.key.as_ref().map(|(column, _)| column)
            {
                continue;
            }
            let same = rows::compare_keys(lock, call, write, write_call).filter(|same| {
                !*same
                    || (rows::stable_prelock_key(lock, call)
                        && rows::stable_prelock_key(write, write_call))
            });
            matches |= same_relation == Some(true) && same == Some(true);
            unknown_key |= same_relation == Some(true) && same.is_none();
            unknown_relation |= same_relation.is_none() && same != Some(false);
        }
        if let (true, Some(_), Some(write_tx)) = (matches, lock_tx, write_tx) {
            if lock.relation.as_deref().is_some_and(|relation| {
                later_access
                    .get_or_init(|| later_access::LaterAccess::collect(calls, grouped))
                    .may_follow(write_tx, write_call.execution_range.1, relation)
            }) {
                continue;
            }
            if let Err(reason) = unique::lock_scope(
                catalog,
                call,
                lock,
                unresolved,
                *only,
                write_facts.conditional_write_unique,
            ) {
                output.unresolved(&site, reason);
                continue;
            }
            if !write_facts.conditional_write_unique
                && !unique::key(catalog, call, lock, unresolved)
            {
                output.unresolved(&site, "cannot prove the pre-lock predicate identifies one row; provide schemaCatalogPath with a matching unique key");
                continue;
            }
            output.emit(
                &site,
                super::super::REDUNDANT_PRELOCK,
                lock.relation.as_deref(),
                lock.key.as_ref().map(|(_, value)| value.as_str()),
                "the following conditional write decides whether to change the locked row; remove the unused pre-lock",
            );
        } else if matches || unknown_key {
            output.unresolved(
                &site,
                "cannot prove the pre-lock and conditional write share a transaction and immutable row key",
            );
        } else if unknown_relation {
            output.unresolved(
                &site,
                "cannot prove the pre-lock and conditional write target the same relation; qualify both relation names consistently",
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
