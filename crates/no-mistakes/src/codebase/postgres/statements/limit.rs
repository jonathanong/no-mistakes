//! `LIMIT` / `FETCH FIRST` facts, shared by every rule that reads a query's row cap.
use sqlparser::ast::{Expr, LimitClause, Query, Value};

/// True when the query itself caps its rows: `LIMIT n` (a literal or a placeholder),
/// MySQL's `LIMIT offset, n`, or `FETCH FIRST`. `LIMIT NULL` and `LIMIT ALL` do not cap.
pub(super) fn is_limited(query: &Query) -> bool {
    if query.fetch.is_some() {
        return true;
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
