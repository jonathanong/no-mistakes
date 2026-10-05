//! Columns a locked relation has pinned to one bound value.
//!
//! A catalog unique key whose columns are all pinned bounds the lock to one row, however
//! many `IN` / `= ANY` filters sit beside it. Only top-level `AND` conjuncts count: an
//! equality inside `OR`, `NOT`, or a subquery does not bound the result.

use super::relations::Relation;
use crate::codebase::postgres::catalog::normalize_table_name;
use sqlparser::ast::{BinaryOperator, Expr, Ident};

pub(super) fn pinned_columns(
    all: &[Relation],
    relation: &Relation,
    selection: Option<&Expr>,
) -> Vec<String> {
    // A self-join cannot tell which alias a pin belongs to, so it proves nothing.
    let repeated = all
        .iter()
        .filter(|other| other.table == relation.table)
        .count()
        > 1;
    let Some(selection) = selection.filter(|_| !repeated) else {
        return Vec::new();
    };
    let mut columns = Vec::new();
    collect(selection, relation, all.len() == 1, &mut columns);
    columns.sort();
    columns.dedup();
    columns
}

fn collect(expr: &Expr, relation: &Relation, sole: bool, out: &mut Vec<String>) {
    match expr {
        Expr::Nested(inner) => collect(inner, relation, sole, out),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            collect(left, relation, sole, out);
            collect(right, relation, sole, out);
        }
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        } => {
            let column = if is_bound_value(right) {
                column_of(left, relation, sole)
            } else if is_bound_value(left) {
                column_of(right, relation, sole)
            } else {
                None
            };
            out.extend(column);
        }
        _ => {}
    }
}

/// A literal or `$n` placeholder, possibly cast: one value for the whole statement.
fn is_bound_value(expr: &Expr) -> bool {
    match expr {
        Expr::Value(_) => true,
        Expr::Nested(inner) | Expr::Cast { expr: inner, .. } => is_bound_value(inner),
        _ => false,
    }
}

fn column_of(expr: &Expr, relation: &Relation, sole: bool) -> Option<String> {
    match expr {
        Expr::Nested(inner) => column_of(inner, relation, sole),
        // An unqualified column is only attributable when the FROM has one relation.
        Expr::Identifier(ident) => sole.then(|| normalize(ident)),
        Expr::CompoundIdentifier(parts) => {
            let (column, qualifier) = parts.split_last()?;
            let qualifier = qualifier
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(".");
            relation
                .names
                .contains(&normalize_table_name(&qualifier))
                .then(|| normalize(column))
        }
        _ => None,
    }
}

fn normalize(ident: &Ident) -> String {
    normalize_table_name(&ident.to_string())
}
