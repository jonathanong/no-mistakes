//! `LIMIT` / `FETCH FIRST` facts, shared by every rule that reads a query's row cap.
mod zero;
pub(super) use super::tokens::Tokens;
use super::value::is_placeholder_ident;
use super::SqlLimitValue;
use crate::codebase::postgres::idents::{ident_key, object_name_ident};
use crate::codebase::postgres::numeric_literal::integer as numeric_literal;
use sqlparser::ast::{
    Expr, FunctionArg, FunctionArgExpr, FunctionArguments, LimitClause, Query, SetExpr, Spanned,
    Value,
};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Span, Token};

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
        // Only the functions that return NULL for nothing but NULL arguments: `NULLIF(1, 1)`,
        // like any function that can produce NULL from fixed inputs, is `LIMIT ALL`.
        Expr::Function(function) => {
            let pick = object_name_ident(&function.name).is_some_and(|ident| {
                ["coalesce", "least", "greatest"].contains(&ident_key(ident).as_str())
            });
            pick && match &function.args {
                FunctionArguments::List(list) => list.args.iter().all(|arg| match arg {
                    FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => is_fixed(expr),
                    _ => false,
                }),
                _ => false,
            }
        }
        _ => false,
    }
}

/// A literal zero row cap prevents even non-streaming set-operation inputs from running.
pub(super) fn is_zero_limited(query: &Query) -> bool {
    let count = if let Some(fetch) = &query.fetch {
        if fetch.with_ties || fetch.percent {
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
pub(super) fn limit_site(query: &Query, tokens: &Tokens) -> Option<LimitSite> {
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
                fetch_keyword(query, tokens).unwrap_or_else(|| start(query.span())),
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

/// Where this query's `FETCH` keyword starts: the first one after everything the query writes
/// before it, so a `FETCH` inside a subquery or a comment is never mistaken for it.
fn fetch_keyword(query: &Query, tokens: &Tokens) -> Option<(usize, usize)> {
    let spans = [
        Some(query.body.span()),
        query.order_by.as_ref().map(Spanned::span),
        query.limit_clause.as_ref().map(Spanned::span),
    ];
    let after = spans.into_iter().flatten().map(end).max()?;
    // An implicit nested FETCH has an empty AST span, including when it ends an OFFSET
    // subquery. Keep token nesting as well as spans so that clause cannot locate this query.
    // A parenthesized body begins before its inner query's span. Account for those
    // opening parentheses before scanning from the body start.
    let body_start = start(query.body.span());
    let mut depth = 0usize;
    let mut body = query.body.as_ref();
    while let SetExpr::Query(inner) = body {
        depth += 1;
        body = inner.body.as_ref();
    }
    // sqlparser does not give TABLE bodies a source span. The token scan cannot
    // anchor its nesting to this query, so retain the span-only lookup in that case.
    if matches!(body, SetExpr::Table(_)) {
        return tokens.all().iter().find_map(|token| match &token.token {
            Token::Word(word) if word.keyword == Keyword::FETCH && start(token.span) >= after => {
                Some(start(token.span))
            }
            _ => None,
        });
    }
    tokens
        .all()
        .iter()
        .filter(|token| start(token.span) >= body_start)
        .find_map(|token| match &token.token {
            Token::LParen => {
                depth += 1;
                None
            }
            Token::RParen => {
                depth = depth.saturating_sub(1);
                None
            }
            Token::Word(word)
                if word.keyword == Keyword::FETCH && depth == 0 && start(token.span) >= after =>
            {
                Some(start(token.span))
            }
            _ => None,
        })
}

fn start(span: Span) -> (usize, usize) {
    (span.start.line as usize, span.start.column as usize)
}

fn end(span: Span) -> (usize, usize) {
    (span.end.line as usize, span.end.column as usize)
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
mod tests;
