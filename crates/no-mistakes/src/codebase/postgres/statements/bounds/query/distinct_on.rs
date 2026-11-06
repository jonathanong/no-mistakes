use super::super::{pins::Resolver, Scope};
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::value::{is_placeholder_ident_at, PlaceholderPositions};
use crate::codebase::postgres::statements::SqlBoundItem;
use sqlparser::ast::{BinaryOperator, Distinct, Expr, Select};

pub(super) fn fixed(
    select: &Select,
    items: &[SqlBoundItem],
    scope: &Scope,
    positions: PlaceholderPositions<'_>,
) -> bool {
    let Some(Distinct::On(expressions)) = &select.distinct else {
        return false;
    };
    let Some(selection) = select.selection.as_ref() else {
        return false;
    };
    if super::super::aggregate::projection_may_erase_input(select, scope) {
        return false;
    }
    let resolver = Resolver::new(items, scope.names(), positions);
    !expressions.is_empty()
        && expressions.iter().all(|expr| {
            resolver
                .column(expr)
                .is_some_and(|column| fixed_by_where(selection, &column, &resolver, positions))
        })
}

fn fixed_by_where(
    expr: &Expr,
    column: &(usize, String),
    resolver: &Resolver,
    positions: PlaceholderPositions<'_>,
) -> bool {
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            fixed_by_where(left, column, resolver, positions)
                || fixed_by_where(right, column, resolver, positions)
        }
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        } => {
            (resolver.column(left).as_ref() == Some(column) && scalar_constant(right, positions))
                || (resolver.column(right).as_ref() == Some(column)
                    && scalar_constant(left, positions))
        }
        _ => false,
    }
}

fn scalar_constant(expr: &Expr, positions: PlaceholderPositions<'_>) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(_) => true,
        Expr::Identifier(ident) => is_placeholder_ident_at(ident, positions),
        _ => false,
    }
}
