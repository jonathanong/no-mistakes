use super::*;
use crate::codebase::postgres::statements::lock_writes::{CteWriteLink, LockWriteFacts, RowAccess};
mod keys;
use keys::identity;
pub(super) use keys::{compare_keys, proven_key, stable_prelock_key};
pub(super) fn enabled_for(call: &EmbeddedSqlCall, written: bool, outside: bool) -> bool {
    if call.caller_transaction_id.is_some() {
        written
    } else if call.executor_transaction_known {
        if call.transaction_id.is_some() {
            written
        } else {
            outside
        }
    } else {
        written || outside
    }
}
pub(super) fn check_transaction(
    output: &mut Output<'_>,
    site: &Site<'_>,
    lock: &RowAccess,
    writes: &writes::Writes<'_>,
    id: u32,
    unresolved: bool,
    written_by_trigger: bool,
) {
    if unresolved {
        output.unresolved(site, "transaction SQL is unresolved; make every write statically visible before comparing the lock with its write set");
    } else if let Some(relation) = &lock.relation {
        if !written_by_trigger && !writes.matches_upsert(id, relation, lock, site.call) {
            let (candidates, ambiguous) = writes.for_relation(id, relation);
            check(output, site, lock, candidates, relation, ambiguous);
        }
    } else {
        output.unresolved(
            site,
            "locked relation is unresolved; lock a single explicit base relation",
        );
    }
}

pub(super) fn check_membership(
    output: &mut Output<'_>,
    site: &Site<'_>,
    lock: &RowAccess,
    same_statement: bool,
    writes: &writes::Writes<'_>,
    unresolved: &BTreeSet<u32>,
) {
    let call = site.call;
    if same_statement {
        return;
    }
    match call.caller_transaction_id {
        Some(id) if !unresolved.contains(&id) && caller_write(call, lock, writes, id) => {}
        Some(_) => output.unresolved(site, "transaction is owned by the caller and a matching local write could not be proven"),
        None => output.unresolved(site, "transaction membership is unresolved; acquire the lock through a configured transaction factory"),
    }
}

pub(super) fn same_statement(
    call: &EmbeddedSqlCall,
    lock: &RowAccess,
    access: &LockWriteFacts,
    lock_index: usize,
    catalog: Option<&SchemaCatalog>,
) -> bool {
    access
        .linked_cte_locks
        .get(&lock_index)
        .is_some_and(|link| match link {
            CteWriteLink::Complete => true,
            CteWriteLink::RequiresNonNull(column) => catalog.is_some_and(|catalog| {
                catalog.coverage() == crate::codebase::postgres::CatalogCoverage::Complete
                    && lock
                        .relation
                        .as_deref()
                        .is_some_and(|relation| catalog.column_is_not_null(relation, column))
            }),
        })
        || access
            .self_contained_locks
            .get(&lock_index)
            .is_some_and(|range| {
                access.writes[range.clone()].iter().any(|write| {
                    write.relation == lock.relation
                        && (!write.has_predicate
                            || compare_keys(lock, call, write, call) == Some(true))
                })
            })
}

fn caller_write(
    call: &EmbeddedSqlCall,
    lock: &RowAccess,
    writes: &writes::Writes<'_>,
    id: u32,
) -> bool {
    let Some(relation) = &lock.relation else {
        return false;
    };
    let (candidates, ambiguous) = writes.for_relation(id, relation);
    call.execution_awaited
        && !ambiguous
        && candidates.iter().any(|(write, candidate)| {
            candidate.execution_scope == call.execution_scope
                && candidate.execution_range.0 > call.execution_range.1
                && (!write.has_predicate
                    || compare_keys(lock, call, write, candidate) == Some(true))
        })
}

fn check(
    output: &mut Output<'_>,
    site: &Site<'_>,
    lock: &RowAccess,
    candidates: &[(&RowAccess, &EmbeddedSqlCall)],
    relation: &str,
    mut unknown: bool,
) {
    for (write, call) in candidates {
        let Some((write_column, write_key)) = &write.key else {
            if !write.has_predicate {
                return;
            }
            unknown = true;
            continue;
        };
        let Some((lock_column, lock_key)) = &lock.key else {
            unknown |= lock.has_predicate;
            continue;
        };
        if write_column != lock_column {
            continue;
        }
        let write_key = identity(write_key, call);
        let lock_key = identity(lock_key, site.call);
        match (write_key, lock_key) {
            (Some(write), Some(lock)) if write == lock => return,
            (None, _) | (_, None) => unknown = true,
            _ => {}
        }
    }
    if candidates.is_empty() && output.write_set(site, relation) {
        return;
    }
    if unknown {
        output.unresolved(site, "row key identity is unresolved; use the same immutable binding and equality column for the lock and write");
    } else {
        output.emit(site, CHECKS[0], Some(relation), lock.key.as_ref().map(|(_, key)| key.as_str()), &format!("locked row in {relation} is not written by this transaction; lock the written row instead or rely on the foreign key KEY SHARE lock"));
    }
}
pub(super) fn constant_key(
    keys: &[crate::codebase::postgres::statements::lock_writes::AdvisoryKey],
    call: &EmbeddedSqlCall,
) -> Option<String> {
    use crate::codebase::postgres::statements::lock_writes::AdvisoryKey;
    fn resolve(key: &AdvisoryKey, call: &EmbeddedSqlCall) -> Option<String> {
        match key {
            AdvisoryKey::Constant(value) => Some(value.clone()),
            AdvisoryKey::Parameter(value) => identity(value, call)?
                .strip_prefix("literal:")
                .map(str::to_owned),
            AdvisoryKey::Hash(name, args) => Some(format!(
                "{name}({})",
                args.iter()
                    .map(|key| resolve(key, call))
                    .collect::<Option<Vec<_>>>()?
                    .join(", ")
            )),
        }
    }
    keys.iter()
        .map(|key| resolve(key, call))
        .collect::<Option<Vec<_>>>()
        .map(|keys| keys.join(", "))
}
