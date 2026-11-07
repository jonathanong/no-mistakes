use super::*;
use crate::codebase::postgres::statements::lock_writes::RowAccess;
pub(super) fn check(
    output: &mut Output<'_>,
    site: &Site<'_>,
    lock: &RowAccess,
    writes: &[(&RowAccess, &EmbeddedSqlCall)],
    relation: &str,
) {
    let candidates: Vec<_> = writes
        .iter()
        .filter(|(write, _)| write.relation.as_deref() == Some(relation))
        .collect();
    let mut unknown = writes.iter().any(|(write, _)| {
        write.relation.as_deref().is_some_and(|name| {
            name != relation
                && crate::codebase::postgres::idents::relation_suffix_key(name)
                    == crate::codebase::postgres::idents::relation_suffix_key(relation)
        })
    });
    for (write, call) in &candidates {
        let Some((write_column, write_key)) = &write.key else {
            if !write.has_predicate {
                return;
            }
            unknown = true;
            continue;
        };
        let Some((lock_column, lock_key)) = &lock.key else {
            unknown = true;
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

pub(super) fn constant_key(keys: &[String], call: &EmbeddedSqlCall) -> Option<String> {
    keys.iter()
        .map(|key| {
            if key.starts_with('$') {
                identity(key, call)?
                    .strip_prefix("literal:")
                    .map(str::to_owned)
            } else {
                Some(key.clone())
            }
        })
        .collect::<Option<Vec<_>>>()
        .map(|keys| keys.join(", "))
}
