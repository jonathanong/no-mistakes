//! Top-level `AND` equality extraction for pinned single- and joined-relation sources.
use super::super::single_row::is_bind;
use super::{SqlPinnedJoinEquality, SqlPinnedRelation};
use crate::idents::ident_key;
use crate::parse_postgres_expression;
use sqlparser::ast::{BinaryOperator, Expr};

pub(super) struct RelationSpec {
    pub(super) table: String,
    pub(super) qualifier: String,
    pub(super) qualified_table: Option<(String, String)>,
}

pub(super) fn collect_pins(
    expr: &Expr,
    specs: &[RelationSpec],
    binds: &[(u32, u32)],
    relations: &mut [SqlPinnedRelation],
    join_equalities: &mut Vec<SqlPinnedJoinEquality>,
) {
    match expr {
        Expr::Nested(inner) => collect_pins(inner, specs, binds, relations, join_equalities),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            collect_pins(left, specs, binds, relations, join_equalities);
            collect_pins(right, specs, binds, relations, join_equalities);
        }
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        } => match (
            resolve_column(left, specs, binds),
            resolve_column(right, specs, binds),
        ) {
            (Some((index, column)), None) if expr_is_constant(right, binds) => {
                relations[index].columns.push(column);
            }
            (None, Some((index, column))) if expr_is_constant(left, binds) => {
                relations[index].columns.push(column);
            }
            (Some((left_relation, left_column)), Some((right_relation, right_column)))
                if left_relation != right_relation =>
            {
                join_equalities.push(SqlPinnedJoinEquality {
                    left_relation,
                    left_column,
                    right_relation,
                    right_column,
                });
            }
            _ => {}
        },
        _ => {}
    }
}

fn resolve_column(
    expr: &Expr,
    specs: &[RelationSpec],
    binds: &[(u32, u32)],
) -> Option<(usize, String)> {
    match expr {
        Expr::Nested(inner) => resolve_column(inner, specs, binds),
        Expr::Identifier(ident) if !is_bind(ident, binds) && specs.len() == 1 => {
            Some((0, ident.to_string()))
        }
        Expr::CompoundIdentifier(parts) => match parts.as_slice() {
            [owner, column] => {
                let key = ident_key(owner);
                unique_match(specs, |spec| spec.qualifier == key)
                    .map(|index| (index, column.to_string()))
            }
            [schema, table, column] => {
                let key = (ident_key(schema), ident_key(table));
                unique_match(specs, |spec| spec.qualified_table.as_ref() == Some(&key))
                    .map(|index| (index, column.to_string()))
            }
            _ => None,
        },
        _ => None,
    }
}

fn unique_match(
    specs: &[RelationSpec],
    predicate: impl Fn(&RelationSpec) -> bool,
) -> Option<usize> {
    let mut matches = specs
        .iter()
        .enumerate()
        .filter(|(_, spec)| predicate(spec))
        .map(|(index, _)| index);
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

pub(in crate::conflict) fn expr_is_constant(expr: &Expr, binds: &[(u32, u32)]) -> bool {
    match expr {
        Expr::Value(_) => true,
        Expr::Identifier(ident) => is_bind(ident, binds),
        Expr::Nested(inner)
        | Expr::Cast { expr: inner, .. }
        | Expr::UnaryOp { expr: inner, .. } => expr_is_constant(inner, binds),
        _ => false,
    }
}

/// Whether `expression` is a literal or bound parameter (possibly cast), so it has the same
/// value in every row and cannot change the order of rows.
pub fn expression_is_constant(expression: &str) -> bool {
    parse_postgres_expression(expression).is_some_and(|expr| expr_is_constant(&expr, &[]))
}
