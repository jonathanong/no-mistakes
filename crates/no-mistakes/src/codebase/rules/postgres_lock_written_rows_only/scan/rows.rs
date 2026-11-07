use super::*;
use crate::codebase::postgres::statements::lock_writes::RowAccess;
pub(super) fn enabled_for(call: &EmbeddedSqlCall, written: bool, outside: bool) -> bool {
    if call.executor_transaction_known {
        if call.transaction_id.is_some() {
            written
        } else {
            outside
        }
    } else {
        written || outside
    }
}
pub(super) fn check(
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
fn identity(value: &str, call: &EmbeddedSqlCall) -> Option<String> {
    if value.starts_with([':', '?']) {
        return None;
    }
    if let Some(index) = value
        .strip_prefix('$')
        .and_then(|value| value.parse::<usize>().ok())
        .and_then(|index| index.checked_sub(1))
    {
        return call.bound_parameters.get(index)?.clone();
    }
    Some(format!("literal:{value}"))
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
