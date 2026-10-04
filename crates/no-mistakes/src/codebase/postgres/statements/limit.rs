//! `LIMIT` / `FETCH FIRST` facts, shared by every rule that reads a query's row cap.
mod empty;
mod fetch;
mod fixed_count;
use fixed_count::is_fixed_at;
mod zero;
pub(super) use super::tokens::Tokens;
use super::SqlLimitValue;
use crate::codebase::postgres::numeric_literal::integer as numeric_literal;
pub(super) use empty::is_empty_page;
use fetch::fetch_keyword;
pub(super) use fetch::next_table_fetch;
use sqlparser::ast::{Expr, LimitClause, Query, Spanned, Value};
use sqlparser::tokenizer::Span;

/// A query's row cap and where its count is written.
pub(super) struct LimitSite {
    pub(super) value: SqlLimitValue,
    pub(super) line: usize,
    pub(super) column: usize,
}

/// True when the query itself caps its rows: `LIMIT n` (a literal, a bind or an expression of
/// them), MySQL's `LIMIT offset, n`, or `FETCH FIRST n ROWS ONLY`. `LIMIT NULL` and `LIMIT ALL`
/// do not cap, and neither does a count taken from the data (`LIMIT (SELECT count(*) …)`),
/// `FETCH FIRST n ROWS WITH TIES` (every row tied with the last is also returned) or a
/// `PERCENT` count (a share of the rows, not a number).
pub(super) fn is_limited_at(
    query: &Query,
    positions: super::value::PlaceholderPositions<'_>,
) -> bool {
    if let Some(fetch) = &query.fetch {
        // `FETCH FIRST ROW ONLY` writes no count: it takes one row.
        return !fetch.with_ties
            && !fetch.percent
            && fetch
                .quantity
                .as_ref()
                .is_none_or(|expr| is_fixed_at(expr, positions));
    }
    match &query.limit_clause {
        Some(LimitClause::LimitOffset {
            limit: Some(limit), ..
        })
        | Some(LimitClause::OffsetCommaLimit { limit, .. }) => is_fixed_at(limit, positions),
        _ => false,
    }
}

/// A literal zero row cap prevents even non-streaming set-operation inputs from running.
pub(super) fn is_zero_limited(query: &Query) -> bool {
    let count = if let Some(fetch) = &query.fetch {
        // A zero count skips the input before tie handling can occur.
        if fetch.percent {
            return false;
        }
        fetch.quantity.as_ref()
    } else {
        match &query.limit_clause {
            Some(LimitClause::LimitOffset { limit, .. }) => limit.as_ref(),
            Some(LimitClause::OffsetCommaLimit { limit, .. }) => Some(limit),
            _ => None,
        }
    };
    count.is_some_and(zero::is_zero)
}

/// The count a query writes after `LIMIT` or `FETCH FIRST`, whether or not it caps the rows,
/// and where: the count itself, or the `FETCH` keyword when it writes none.
pub(super) fn limit_site_at(
    query: &Query,
    tokens: &Tokens,
    table_fetch: Option<(usize, usize)>,
) -> Option<LimitSite> {
    if let Some(fetch) = &query.fetch {
        // `FETCH FIRST ROW ONLY` takes one row; a percentage is not a row count.
        let value = match (&fetch.quantity, fetch.percent) {
            (_, true) => SqlLimitValue::Other,
            (None, false) => SqlLimitValue::Literal(1),
            (Some(quantity), false) => literal(quantity, tokens),
        };
        return Some(match &fetch.quantity {
            Some(quantity) => site(value, start(quantity.span())),
            None => site(
                value,
                fetch_keyword(query, tokens, table_fetch).unwrap_or_else(|| start(query.span())),
            ),
        });
    }
    match &query.limit_clause {
        Some(LimitClause::LimitOffset {
            limit: Some(limit), ..
        })
        | Some(LimitClause::OffsetCommaLimit { limit, .. })
            if !is_null(limit) =>
        {
            Some(site(literal(limit, tokens), start(limit.span())))
        }
        _ => None,
    }
}

fn start(span: Span) -> (usize, usize) {
    (span.start.line as usize, span.start.column as usize)
}

fn site(value: SqlLimitValue, (line, column): (usize, usize)) -> LimitSite {
    LimitSite {
        value,
        line,
        column,
    }
}

fn literal(expr: &Expr, tokens: &Tokens<'_>) -> SqlLimitValue {
    match expr {
        Expr::Nested(inner) => literal(inner, tokens),
        Expr::Value(value) => match &value.value {
            Value::Number(text, _) => {
                numeric_literal(text).map_or(SqlLimitValue::Other, SqlLimitValue::Literal)
            }
            Value::HexStringLiteral(_) => tokens
                .source_at(value.span())
                .filter(|text| {
                    text.get(..2)
                        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("0x"))
                })
                .and_then(|text| numeric_literal(text).ok())
                .map_or(SqlLimitValue::Other, SqlLimitValue::Literal),
            _ => SqlLimitValue::Other,
        },
        _ => SqlLimitValue::Other,
    }
}

fn is_null(expr: &Expr) -> bool {
    match expr {
        Expr::Nested(inner) => is_null(inner),
        Expr::Value(value) => matches!(value.value, Value::Null),
        _ => false,
    }
}
#[cfg(test)]
mod fetch_tests;
#[cfg(test)]
mod tests;
