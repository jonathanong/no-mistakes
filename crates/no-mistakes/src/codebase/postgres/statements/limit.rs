//! `LIMIT` / `FETCH FIRST` facts, shared by every rule that reads a query's row cap.
use sqlparser::ast::{Expr, LimitClause, Query, Value};

/// True when the query itself caps its rows: `LIMIT n` (a literal or a placeholder),
/// MySQL's `LIMIT offset, n`, or `FETCH FIRST n ROWS ONLY`. `LIMIT NULL` and `LIMIT ALL` do not
/// cap, and neither does `FETCH FIRST n ROWS WITH TIES` (every row tied with the last is also
/// returned) or a `PERCENT` count (a share of the rows, not a number).
pub(super) fn is_limited(query: &Query) -> bool {
    if let Some(fetch) = &query.fetch {
        return !fetch.with_ties && !fetch.percent;
    }
    match &query.limit_clause {
        Some(LimitClause::LimitOffset {
            limit: Some(limit), ..
        })
        | Some(LimitClause::OffsetCommaLimit { limit, .. }) => !is_null(limit),
        _ => false,
    }
}

fn is_null(expr: &Expr) -> bool {
    match expr {
        Expr::Nested(inner) => is_null(inner),
        Expr::Value(value) => matches!(value.value, Value::Null),
        _ => false,
    }
}
