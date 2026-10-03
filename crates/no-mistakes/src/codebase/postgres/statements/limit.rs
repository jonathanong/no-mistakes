//! `LIMIT` / `FETCH FIRST` facts, shared by every rule that reads a query's row cap.
use super::value::is_placeholder_ident;
use sqlparser::ast::{
    Expr, FunctionArg, FunctionArgExpr, FunctionArguments, LimitClause, Query, Value,
};

/// True when the query itself caps its rows: `LIMIT n` (a literal, a bind or an expression of
/// them), MySQL's `LIMIT offset, n`, or `FETCH FIRST n ROWS ONLY`. `LIMIT NULL` and `LIMIT ALL`
/// do not cap, and neither does a count taken from the data (`LIMIT (SELECT count(*) …)`),
/// `FETCH FIRST n ROWS WITH TIES` (every row tied with the last is also returned) or a
/// `PERCENT` count (a share of the rows, not a number).
pub(super) fn is_limited(query: &Query) -> bool {
    if let Some(fetch) = &query.fetch {
        // `FETCH FIRST ROW ONLY` writes no count: it takes one row.
        return !fetch.with_ties && !fetch.percent && fetch.quantity.as_ref().is_none_or(is_fixed);
    }
    match &query.limit_clause {
        Some(LimitClause::LimitOffset {
            limit: Some(limit), ..
        })
        | Some(LimitClause::OffsetCommaLimit { limit, .. }) => is_fixed(limit),
        _ => false,
    }
}

/// A count the statement text or its caller decides: a literal, a bind (`$1`, or an
/// interpolation recovered from a template literal) or an expression of them. NULL means no
/// limit, and a subquery or a column can return any number.
fn is_fixed(expr: &Expr) -> bool {
    match expr {
        Expr::Nested(inner) => is_fixed(inner),
        Expr::Value(value) => !matches!(value.value, Value::Null),
        Expr::Identifier(ident) => is_placeholder_ident(&ident.value),
        Expr::Cast { expr, .. } | Expr::UnaryOp { expr, .. } => is_fixed(expr),
        Expr::BinaryOp { left, right, .. } => is_fixed(left) && is_fixed(right),
        Expr::Function(function) => match &function.args {
            FunctionArguments::List(list) => list.args.iter().all(|arg| match arg {
                FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => is_fixed(expr),
                _ => false,
            }),
            _ => false,
        },
        _ => false,
    }
}
