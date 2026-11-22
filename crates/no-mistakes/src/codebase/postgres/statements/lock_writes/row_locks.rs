use super::{
    helpers::{access, generated_name, qualifier},
    scope, RowAccess,
};
use crate::codebase::postgres::idents::object_name_identity;
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use sqlparser::ast::{LockType, Query, SetExpr};
use std::collections::BTreeSet;

mod explicit;
use explicit::collect_explicit;

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;

pub(super) fn collect_advanced(
    query: &Query,
    positions: PlaceholderPositions<'_>,
    scopes: &[BTreeSet<String>],
    locks: &mut Vec<RowAccess>,
    mut share_relations: Option<&mut Vec<Option<String>>>,
    top_level: bool,
) {
    let rows = if query.locks.iter().all(|lock| lock.of.is_some()) {
        collect_explicit(query, positions, share_relations.is_some())
    } else {
        let row = collect_legacy(query, positions, true);
        let shared = if row.relation.is_some() {
            query
                .locks
                .iter()
                .all(|lock| lock.lock_type == LockType::Share)
        } else {
            has_share(query)
        };
        vec![(row, shared)]
    };
    for (mut lock, shared) in rows {
        scope::hide_cte(&mut lock, scopes);
        if shared {
            if let Some(relations) = share_relations.as_deref_mut() {
                relations.push(if top_level {
                    lock.relation.clone()
                } else {
                    None
                });
            }
        }
        locks.push(lock);
    }
}

pub(super) fn collect_legacy(
    query: &Query,
    positions: PlaceholderPositions<'_>,
    allow: bool,
) -> RowAccess {
    if !allow {
        return unresolved();
    }
    let SetExpr::Select(select) = query.body.as_ref() else {
        return unresolved();
    };
    match select.from.as_slice() {
        [from] if from.joins.is_empty() => {
            let row = access(&from.relation, select.selection.as_ref(), positions);
            let valid = query.locks.iter().all(|lock| {
                lock.of.as_ref().is_none_or(|target| {
                    !generated_name(target, positions)
                        && qualifier(&from.relation)
                            .is_some_and(|name| name == object_name_identity(target))
                })
            });
            if valid {
                row
            } else {
                unresolved()
            }
        }
        _ => unresolved(),
    }
}

fn has_share(query: &Query) -> bool {
    query
        .locks
        .iter()
        .any(|lock| lock.lock_type == LockType::Share)
}

fn unresolved() -> RowAccess {
    RowAccess {
        relation: None,
        key: None,
        recovered_key: None,
        key_identity_unproven: false,
        has_predicate: false,
    }
}
