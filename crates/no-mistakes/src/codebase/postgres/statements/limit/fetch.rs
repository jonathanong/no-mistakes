use super::Tokens;
use sqlparser::ast::{Query, SetExpr, Spanned};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Span, Token};

/// Where this query's `FETCH` keyword starts: after clauses it writes, not nested queries/comments.
pub(super) fn fetch_keyword(
    query: &Query,
    tokens: &Tokens,
    table_fetch: Option<(usize, usize)>,
) -> Option<(usize, usize)> {
    let body_span = query.body.span();
    let spans = [
        Some(body_span),
        query.order_by.as_ref().map(Spanned::span),
        query.limit_clause.as_ref().map(Spanned::span),
    ];
    let after = spans
        .into_iter()
        .flatten()
        .map(end)
        .fold(end(body_span), std::cmp::max);
    // A parenthesized body begins before its inner query's span, so initialize the token depth
    // with those wrappers before scanning from the body start.
    let body_start = start(body_span);
    let mut depth = leading_parenthesized_queries(query.body.as_ref());
    let mut body = query.body.as_ref();
    while let SetExpr::Query(inner) = body {
        body = inner.body.as_ref();
    }
    // TABLE bodies have no source span; the ordered caller supplies their matching token.
    if matches!(body, SetExpr::Table(_)) {
        return table_fetch;
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

/// Locate each spanless TABLE query's FETCH in visitor order; repeated names keep their occurrence.
pub(in crate::codebase::postgres::statements) fn next_table_fetch(
    query: &Query,
    tokens: &Tokens,
    cursor: &mut usize,
) -> Option<(usize, usize)> {
    let mut body = query.body.as_ref();
    while let SetExpr::Query(inner) = body {
        body = inner.body.as_ref();
    }
    let SetExpr::Table(table) = body else {
        return None;
    };
    let table_name = table.table_name.as_deref()?;
    let all = tokens.all();
    let (_, relation_end) = (*cursor..all.len()).find_map(|table_index| {
        if !keyword_at(all, table_index, Keyword::TABLE) {
            return None;
        }
        let name_index = next_token(all, table_index + 1).unwrap_or(all.len());
        let (relation_end, matches_name) = if let Some(schema) = table.schema_name.as_deref() {
            let period_index = next_token(all, name_index + 1).unwrap_or(all.len());
            let table_name_index = next_token(all, period_index + 1).unwrap_or(all.len());
            (
                table_name_index,
                word_at(all, name_index, schema)
                    && matches!(
                        all.get(period_index).map(|token| &token.token),
                        Some(Token::Period)
                    )
                    && word_at(all, table_name_index, table_name),
            )
        } else {
            (name_index, word_at(all, name_index, table_name))
        };
        matches_name.then_some((table_index, relation_end))
    })?;
    if query.fetch.is_none() {
        *cursor = relation_end + 1;
        return None;
    }
    let fetch_index =
        (relation_end + 1..all.len()).find(|index| keyword_at(all, *index, Keyword::FETCH))?;
    *cursor = fetch_index + 1;
    Some(start(all[fetch_index].span))
}

fn keyword_at(
    tokens: &[sqlparser::tokenizer::TokenWithSpan],
    index: usize,
    keyword: Keyword,
) -> bool {
    matches!(tokens.get(index).map(|token| &token.token), Some(Token::Word(word)) if word.keyword == keyword)
}

fn word_at(tokens: &[sqlparser::tokenizer::TokenWithSpan], index: usize, value: &str) -> bool {
    matches!(tokens.get(index).map(|token| &token.token), Some(Token::Word(word)) if word.value == value)
}

fn next_token(tokens: &[sqlparser::tokenizer::TokenWithSpan], from: usize) -> Option<usize> {
    (from..tokens.len()).find(|index| !matches!(tokens[*index].token, Token::Whitespace(_)))
}

/// Parenthesized set-expression wrappers put initial body tokens inside their parentheses.
fn leading_parenthesized_queries(body: &SetExpr) -> usize {
    match body {
        SetExpr::Query(inner) => 1 + leading_parenthesized_queries(&inner.body),
        SetExpr::SetOperation { left, .. } => leading_parenthesized_queries(left),
        _ => 0,
    }
}

fn start(span: Span) -> (usize, usize) {
    (span.start.line as usize, span.start.column as usize)
}

fn end(span: Span) -> (usize, usize) {
    (span.end.line as usize, span.end.column as usize)
}
