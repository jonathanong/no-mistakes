use sqlparser::ast::{Expr, Spanned};

pub(super) fn line_of(expr: &Expr) -> usize {
    expr.span().start.line.max(1) as usize
}
