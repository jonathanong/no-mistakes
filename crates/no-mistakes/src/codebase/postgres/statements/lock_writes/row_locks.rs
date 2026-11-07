use super::{
    RowAccess,
    helpers::{access, generated_name, qualifier},
};
use crate::codebase::postgres::idents::{ident_key, object_name_identity};
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use sqlparser::ast::{
    BinaryOperator, Expr, ObjectName, ObjectNamePart, Query, SetExpr, TableFactor,
};

pub(super) fn collect(
    query: &Query,
    positions: PlaceholderPositions<'_>,
    advanced: bool,
    allow_legacy: bool,
) -> Vec<RowAccess> {
    if advanced && query.locks.iter().all(|lock| lock.of.is_some()) {
        return collect_explicit(query, positions);
    }
    if !allow_legacy {
        return vec![unresolved()];
    }
    collect_legacy(query, positions)
}

fn collect_explicit(query: &Query, positions: PlaceholderPositions<'_>) -> Vec<RowAccess> {
    let SetExpr::Select(select) = query.body.as_ref() else {
        return vec![unresolved()];
    };
    let mut locks = Vec::new();
    for lock in &query.locks {
        let Some(target) = lock
            .of
            .as_ref()
            .filter(|target| !generated_name(target, positions))
        else {
            return vec![unresolved()];
        };
        let matches = select
            .from
            .iter()
            .flat_map(|from| {
                std::iter::once(&from.relation).chain(from.joins.iter().map(|join| &join.relation))
            })
            .filter(|factor| target_matches(target, factor))
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return vec![unresolved()];
        }
        let mut row = access(matches[0], select.selection.as_ref(), positions);
        if query.locks.len() > 1 && row.key.is_none() {
            row.key = select
                .selection
                .as_ref()
                .and_then(|predicate| conjunctive_key(matches[0], predicate, positions));
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
    match predicate {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => conjunctive_key(factor, left, positions)
            .or_else(|| conjunctive_key(factor, right, positions)),
        expression => access(factor, Some(expression), positions).key,
    }
}

fn collect_legacy(query: &Query, positions: PlaceholderPositions<'_>) -> Vec<RowAccess> {
    let SetExpr::Select(select) = query.body.as_ref() else {
        return vec![unresolved()];
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
            vec![if valid { row } else { unresolved() }]
        }
        _ => vec![unresolved()],
    }
}

fn target_matches(target: &ObjectName, factor: &TableFactor) -> bool {
    let TableFactor::Table {
        name,
        alias,
        args: None,
        ..
    } = factor
    else {
        return false;
    };
    if let Some(alias) = alias {
        return ident_key(&alias.name) == object_name_identity(target);
    }
    let target_name = object_name_identity(target);
    object_name_identity(name) == target_name
        || name
            .0
            .last()
            .and_then(ObjectNamePart::as_ident)
            .is_some_and(|ident| ident_key(ident) == target_name)
}

fn unresolved() -> RowAccess {
    RowAccess {
        relation: None,
        key: None,
        has_predicate: false,
    }
}
