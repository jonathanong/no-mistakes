use super::{
    helpers::{access, generated_name, qualifier},
    scope, RowAccess,
};
use crate::codebase::postgres::idents::{ident_key, object_name_identity, relation_part_key};
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use sqlparser::ast::{
    BinaryOperator, Expr, LockType, ObjectNamePart, Query, Select, SetExpr, TableFactor,
};
use std::collections::{BTreeMap, BTreeSet};

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
        collect_explicit(query, positions)
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

pub(super) fn collect_explicit(
    query: &Query,
    positions: PlaceholderPositions<'_>,
) -> Vec<(RowAccess, bool)> {
    let SetExpr::Select(select) = query.body.as_ref() else {
        return vec![(unresolved(), has_share(query))];
    };
    let relations = relation_index(select);
    let mut locks = Vec::new();
    for lock in &query.locks {
        let Some(target) = lock
            .of
            .as_ref()
            .filter(|target| !generated_name(target, positions))
        else {
            return vec![(unresolved(), has_share(query))];
        };
        let target = object_name_identity(target);
        let Some(factors) = relations.get(&target).filter(|factors| factors.len() == 1) else {
            return vec![(unresolved(), has_share(query))];
        };
        let factor = factors[0];
        let mut row = access(factor, select.selection.as_ref(), positions);
        if query.locks.len() > 1 && row.key.is_none() {
            if let Some(keyed) = select
                .selection
                .as_ref()
                .and_then(|predicate| conjunctive_key(factor, predicate, positions))
            {
                row.key = keyed.key;
                row.recovered_key = keyed.recovered_key;
                row.key_identity_unproven |= keyed.key_identity_unproven;
            }
        }
        let shared = lock.lock_type == LockType::Share;
        if let Some((_, prior_shared)) = locks.iter_mut().find(|(candidate, _)| *candidate == row) {
            *prior_shared &= shared;
        } else {
            locks.push((row, shared));
        }
    }
    locks
}

fn conjunctive_key(
    factor: &TableFactor,
    predicate: &Expr,
    positions: PlaceholderPositions<'_>,
) -> Option<RowAccess> {
    match super::unwrap_expr(predicate) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => conjunctive_key(factor, left, positions)
            .or_else(|| conjunctive_key(factor, right, positions)),
        expression => {
            let row = access(factor, Some(expression), positions);
            row.key.is_some().then_some(row)
        }
    }
}

fn relation_index(select: &Select) -> BTreeMap<String, Vec<&TableFactor>> {
    let mut index = BTreeMap::<String, Vec<&TableFactor>>::new();
    for from in &select.from {
        for factor in
            std::iter::once(&from.relation).chain(from.joins.iter().map(|join| &join.relation))
        {
            let TableFactor::Table {
                name,
                alias,
                args: None,
                ..
            } = factor
            else {
                continue;
            };
            if let Some(alias) = alias {
                index
                    .entry(normalized_ident(&alias.name))
                    .or_default()
                    .push(factor);
                continue;
            }
            let table_key = object_name_identity(name);
            index.entry(table_key.clone()).or_default().push(factor);
            if let Some(leaf) = name.0.last().and_then(ObjectNamePart::as_ident) {
                let leaf_key = normalized_ident(leaf);
                if leaf_key != table_key {
                    index.entry(leaf_key).or_default().push(factor);
                }
            }
        }
    }
    index
}

fn normalized_ident(ident: &sqlparser::ast::Ident) -> String {
    relation_part_key(&ident_key(ident))
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
