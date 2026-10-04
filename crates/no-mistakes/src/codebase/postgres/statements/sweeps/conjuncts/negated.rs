use super::{Cursor, SqlCursorBound};
use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{Expr, UnaryOperator};

/// Resolve the complete NOT chain before judging a NULL-switchable cursor.
pub(super) fn of(expr: &Expr, resolve: impl FnOnce(&Expr) -> Option<Cursor>) -> Option<Cursor> {
    let mut odd = true;
    let mut inner = unwrap_expr(expr);
    while let Expr::UnaryOp {
        op: UnaryOperator::Not,
        expr,
    } = inner
    {
        odd = !odd;
        inner = unwrap_expr(expr);
    }
    let mut cursor = resolve(inner)?;
    if !odd {
        return Some(cursor);
    }
    // Negating a NULL-switchable OR is not merely reversing its comparison.
    if cursor.optional {
        return None;
    }
    cursor.bound = match cursor.bound {
        SqlCursorBound::Lower => SqlCursorBound::Upper,
        SqlCursorBound::Upper => SqlCursorBound::Lower,
    };
    Some(cursor)
}
