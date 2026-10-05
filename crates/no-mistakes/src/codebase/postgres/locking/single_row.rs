//! Columns a `SELECT`'s base relations have pinned, by value or through a join.
//!
//! A catalog unique key whose columns are all pinned bounds a relation to one row, however
//! many `IN` / `= ANY` filters sit beside it. Only top-level `AND` conjuncts of `WHERE` and of
//! an inner join's `ON` count: an equality inside `OR`, `NOT`, an outer join, or a subquery
//! does not bound the result.

use super::relations::Relation;
use crate::codebase::postgres::catalog::normalize_table_name;
use crate::codebase::postgres::statements::value::is_placeholder_ident_at;
use sqlparser::ast::{BinaryOperator, Expr, Ident, JoinConstraint, JoinOperator, Select};
use std::collections::BTreeMap;

/// `left_table.left_column = right_table.right_column`, an inner-join or `WHERE` equality
/// between two distinct base relations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinEquality {
    pub left_table: String,
    pub left_column: String,
    pub right_table: String,
    pub right_column: String,
}

#[derive(Default)]
pub(super) struct Pins {
    /// Columns of each base relation pinned to one bound value.
    pub(super) bound: BTreeMap<String, Vec<String>>,
    pub(super) joins: Vec<JoinEquality>,
}

enum Side {
    Bound,
    Column(usize, String),
    Other,
}

pub(super) fn analyze(all: &[Relation], select: &Select, positions: &[(u32, u32)]) -> Pins {
    let mut pins = Pins::default();
    for relation in all.iter().filter_map(|relation| relation.table.as_ref()) {
        pins.bound.entry(relation.clone()).or_default();
    }
    let mut conjuncts = Vec::new();
    conjuncts_of(select.selection.as_ref(), &mut conjuncts);
    for table in &select.from {
        for join in &table.joins {
            if let JoinOperator::Join(JoinConstraint::On(on))
            | JoinOperator::Inner(JoinConstraint::On(on)) = &join.join_operator
            {
                conjuncts_of(Some(on), &mut conjuncts);
            }
        }
    }
    for (left, right) in conjuncts {
        let (left, right) = (side(left, all, positions), side(right, all, positions));
        match (left, right) {
            (Side::Column(index, column), Side::Bound)
            | (Side::Bound, Side::Column(index, column)) => {
                if let Some(table) = &all[index].table {
                    pins.bound.entry(table.clone()).or_default().push(column);
                }
            }
            (Side::Column(a, a_column), Side::Column(b, b_column)) if a != b => {
                if let (Some(left_table), Some(right_table)) = (&all[a].table, &all[b].table) {
                    pins.joins.push(JoinEquality {
                        left_table: left_table.clone(),
                        left_column: a_column,
                        right_table: right_table.clone(),
                        right_column: b_column,
                    });
                }
            }
            _ => {}
        }
    }
    for columns in pins.bound.values_mut() {
        columns.sort();
        columns.dedup();
    }
    pins
}

/// The `left = right` conjuncts reachable through top-level `AND` only.
fn conjuncts_of<'a>(expr: Option<&'a Expr>, out: &mut Vec<(&'a Expr, &'a Expr)>) {
    match expr {
        Some(Expr::Nested(inner)) => conjuncts_of(Some(inner), out),
        Some(Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        }) => {
            conjuncts_of(Some(left), out);
            conjuncts_of(Some(right), out);
        }
        Some(Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        }) => out.push((left, right)),
        _ => {}
    }
}

fn side(expr: &Expr, all: &[Relation], positions: &[(u32, u32)]) -> Side {
    match expr {
        Expr::Nested(inner) => side(inner, all, positions),
        // A literal or `$n`, possibly cast: one value for the whole statement.
        Expr::Value(_) => Side::Bound,
        Expr::Cast { expr: inner, .. } => match side(inner, all, positions) {
            Side::Bound => Side::Bound,
            _ => Side::Other,
        },
        // A recovered interpolation is exactly one bound value; user-authored text that merely
        // spells the marker is a column.
        Expr::Identifier(ident) if is_placeholder_ident_at(ident, Some(positions)) => Side::Bound,
        // An unqualified column is only attributable when the FROM has one relation.
        Expr::Identifier(ident) if all.len() == 1 => relation_column(all, 0, ident),
        Expr::CompoundIdentifier(parts) => qualified(parts, all),
        _ => Side::Other,
    }
}

fn qualified(parts: &[Ident], all: &[Relation]) -> Side {
    let Some((column, qualifier)) = parts.split_last() else {
        return Side::Other;
    };
    let qualifier = qualifier
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(".");
    let qualifier = normalize_table_name(&qualifier);
    let mut matches = all
        .iter()
        .enumerate()
        .filter(|(_, relation)| relation.names.contains(&qualifier));
    match (matches.next(), matches.next()) {
        (Some((index, _)), None) => relation_column(all, index, column),
        _ => Side::Other,
    }
}

fn relation_column(all: &[Relation], index: usize, column: &Ident) -> Side {
    let Some(table) = &all[index].table else {
        return Side::Other;
    };
    // A self-join cannot tell which alias a pin belongs to, so it proves nothing.
    let repeated = all
        .iter()
        .filter(|other| other.table.as_ref() == Some(table))
        .count()
        > 1;
    if repeated {
        Side::Other
    } else {
        Side::Column(index, normalize_table_name(&column.to_string()))
    }
}
