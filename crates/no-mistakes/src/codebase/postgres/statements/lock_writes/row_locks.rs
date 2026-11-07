use super::{
    RowAccess,
    helpers::{access, generated_name, qualifier},
};
use crate::codebase::postgres::idents::{ident_key, object_name_identity, relation_part_key};
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use sqlparser::ast::{BinaryOperator, Expr, ObjectNamePart, Query, Select, SetExpr, TableFactor};
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

pub(super) fn collect_advanced(
    query: &Query,
    positions: PlaceholderPositions<'_>,
) -> Vec<RowAccess> {
    if query.locks.iter().all(|lock| lock.of.is_some()) {
        return collect_explicit(query, positions);
    }
    vec![collect_legacy(query, positions, true)]
}

pub(super) fn collect_explicit(
    query: &Query,
    positions: PlaceholderPositions<'_>,
) -> Vec<RowAccess> {
    let SetExpr::Select(select) = query.body.as_ref() else {
        return vec![unresolved()];
    };
    let relations = relation_index(select);
    let mut locks = Vec::new();
    for lock in &query.locks {
        let Some(target) = lock
            .of
            .as_ref()
            .filter(|target| !generated_name(target, positions))
        else {
            return vec![unresolved()];
        };
        let target = object_name_identity(target);
        let Some(factors) = relations.get(&target).filter(|factors| factors.len() == 1) else {
            return vec![unresolved()];
        };
        let factor = factors[0];
        let mut row = access(factor, select.selection.as_ref(), positions);
        if query.locks.len() > 1 && row.key.is_none() {
            row.key = select
                .selection
                .as_ref()
                .and_then(|predicate| conjunctive_key(factor, predicate, positions));
        }
        if !locks.contains(&row) {
            locks.push(row);
        }
    }
    locks
}

fn conjunctive_key(
    factor: &TableFactor,
    predicate: &Expr,
    positions: PlaceholderPositions<'_>,
) -> Option<(String, String)> {
    match super::unwrap_expr(predicate) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => conjunctive_key(factor, left, positions)
            .or_else(|| conjunctive_key(factor, right, positions)),
        expression => access(factor, Some(expression), positions).key,
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
            if valid { row } else { unresolved() }
        }
        _ => unresolved(),
    }
}

fn unresolved() -> RowAccess {
    RowAccess {
        relation: None,
        key: None,
        has_predicate: false,
    }
}
