//! `LIMIT` / `FETCH FIRST` facts, shared by every rule that reads a query's row cap.
use super::value::is_placeholder_ident;
use super::SqlLimitValue;
use crate::codebase::postgres::idents::{ident_key, object_name_ident};
use sqlparser::ast::{
    Expr, FunctionArg, FunctionArgExpr, FunctionArguments, LimitClause, Query, Spanned, Value,
};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Span, Token, TokenWithSpan, Tokenizer};
use std::cell::OnceCell;

/// A query's row cap and where its count is written.
pub(super) struct LimitSite {
    pub(super) value: SqlLimitValue,
    pub(super) line: usize,
    pub(super) column: usize,
}

/// The tokens of one SQL source, produced on first use: only a `FETCH FIRST ROW ONLY`, which
/// writes no count to point at, needs them to find its `FETCH` keyword.
pub(super) struct Tokens<'a> {
    sql: &'a str,
    tokens: OnceCell<Vec<TokenWithSpan>>,
}

impl<'a> Tokens<'a> {
    pub(super) fn new(sql: &'a str) -> Self {
        Self {
            sql,
            tokens: OnceCell::new(),
        }
    }

    fn all(&self) -> &[TokenWithSpan] {
        self.tokens.get_or_init(|| {
            Tokenizer::new(&PostgreSqlDialect {}, self.sql)
                .tokenize_with_location()
                .unwrap_or_default()
        })
    }
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
            let pick = function.name.0.len() == 1
                && object_name_ident(&function.name).is_some_and(|ident| {
                    ident.quote_style.is_none()
                        && ["coalesce", "least", "greatest"].contains(&ident_key(ident).as_str())
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

/// The count a query writes after `LIMIT` or `FETCH FIRST`, whether or not it caps the rows,
/// and where: the count itself, or the `FETCH` keyword when it writes none.
pub(super) fn limit_site(query: &Query, tokens: &Tokens) -> Option<LimitSite> {
    if let Some(fetch) = &query.fetch {
        // `FETCH FIRST ROW ONLY` takes one row; a percentage is not a row count.
        let value = match (&fetch.quantity, fetch.percent) {
            (_, true) => SqlLimitValue::Other,
            (None, false) => SqlLimitValue::Literal(1),
            (Some(quantity), false) => literal(quantity),
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
            Some(site(literal(limit), start(limit.span())))
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
    tokens.all().iter().find_map(|token| match &token.token {
        Token::Word(word) if word.keyword == Keyword::FETCH && start(token.span) >= after => {
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

fn literal(expr: &Expr) -> SqlLimitValue {
    match expr {
        Expr::Nested(inner) => literal(inner),
        Expr::Value(value) => match &value.value {
            Value::Number(text, _) => {
                numeric_literal(text).map_or(SqlLimitValue::Other, SqlLimitValue::Literal)
            }
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

/// PostgreSQL digit separators are valid only between digits of the literal's radix.
fn numeric_literal(text: &str) -> Result<u64, std::num::ParseIntError> {
    let (digits, radix) = match text.get(..2).map(str::to_ascii_lowercase).as_deref() {
        Some("0x") => (&text[2..], 16),
        Some("0o") => (&text[2..], 8),
        Some("0b") => (&text[2..], 2),
        _ => (text, 10),
    };
    let chars: Vec<char> = digits.chars().collect();
    if chars.iter().enumerate().any(|(index, character)| {
        *character == '_'
            && (index == 0
                || index + 1 == chars.len()
                || !chars[index - 1].is_digit(radix)
                || !chars[index + 1].is_digit(radix))
    }) {
        return u64::from_str_radix("invalid", radix);
    }
    u64::from_str_radix(&digits.replace('_', ""), radix)
}

#[cfg(test)]
mod tests;
