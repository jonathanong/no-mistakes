use sqlparser::ast::Statement;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan, Word};
use std::sync::Arc;

mod chr;
mod chunks;
use chunks::split_statement_tokens;
pub(super) use chunks::{next_non_ws, skip_ws};
mod fallback;
mod recover;
pub(super) mod rewrite;
mod source;
pub(super) use chr::{expand_chr_encoded_sql, rewrite_chr_tokens};
pub(crate) use recover::BodyState as ProceduralBodyState;
pub(crate) use source::LocatedStatement;

use rewrite::{rewrite_drop_index_concurrently, rewrite_referential_set_column_lists};

/// Tokenize, rewrite PG18 virtual generated columns,
/// `ON DELETE` column lists on `SET NULL` / `SET DEFAULT`, and
/// `DROP INDEX CONCURRENTLY`, then parse each statement.
///
/// Unparseable `DO $tag$ … $tag$` statements are peeled so schema DDL inside
/// the body can still parse. Remaining unparseable chunks recover `ALTER TABLE`,
/// `CREATE TABLE`, `CREATE [UNIQUE] INDEX`, and DML after PL/pgSQL wrappers.
pub(super) fn parse_postgres_sql_lenient(sql: &str) -> Vec<Statement> {
    parse_with_sources(sql, false, true, false)
        .into_iter()
        .filter(|located| !located.function_projection)
        .map(|located| located.statement)
        .collect()
}

pub(crate) fn parse_postgres_body_with_function_sources(sql: &str) -> Vec<LocatedStatement> {
    parse_with_sources(sql, true, false, true)
}

pub(crate) fn parse_postgres_sql_with_function_sources(sql: &str) -> Vec<LocatedStatement> {
    parse_with_sources(sql, false, true, false)
}

pub(super) fn parse_postgres_sql_lenient_with_sources(
    sql: &str,
    normalized: &str,
) -> Vec<LocatedStatement> {
    parse_with_normalized_sources(sql, normalized, false, true, false)
}

fn parse_with_sources(
    sql: &str,
    fragment: bool,
    allow_concurrent_detach: bool,
    procedural_body: bool,
) -> Vec<LocatedStatement> {
    let normalized = super::normalize_copy_data(sql);
    parse_with_normalized_sources(
        sql,
        &normalized,
        fragment,
        allow_concurrent_detach,
        procedural_body,
    )
}

fn parse_with_normalized_sources(
    sql: &str,
    normalized: &str,
    fragment: bool,
    allow_concurrent_detach: bool,
    procedural_body: bool,
) -> Vec<LocatedStatement> {
    let separated = super::distinct_group::separate_distinct_grouping(normalized);
    let mut located = super::unicode::tokenize_with_location(&separated, false);
    // Capture recursive headers before other compatibility rewrites can change their heads.
    let mut original_heads = std::collections::BTreeMap::new();
    let recursive_views = super::recursive_view::prepare(&mut located, Some(&mut original_heads));
    let mut located = super::radix_numbers::repair(&located).unwrap_or(located);
    super::normalize_table_queries(&mut located);
    super::ddl_compat::normalize(&mut located);
    let fetch = super::fetch_expression::prepare(&mut located);
    if located.is_empty() {
        return fallback::statements(sql, allow_concurrent_detach, procedural_body);
    }
    let mut tokens = located.iter().map(|token| token.token.clone()).collect();
    rewrite_virtual_generated_columns(&mut tokens);
    rewrite_referential_set_column_lists(&mut tokens);
    rewrite_drop_index_concurrently(&mut tokens);
    let mut statements = recover::parse_chunks_with_function_calls(
        split_statement_tokens(tokens),
        &located,
        allow_concurrent_detach,
        procedural_body,
        &recursive_views,
    );
    recursive_views.restore_statements(&mut statements, &original_heads);
    for statement in &mut statements {
        super::fetch_expression::restore(&mut statement.statement, &fetch);
    }
    if fragment {
        let source: Arc<[TokenWithSpan]> = located.into();
        for statement in &mut statements {
            if statement.source.is_none() {
                statement.source = Some(Arc::clone(&source));
            }
        }
    }
    statements
}

fn rewrite_virtual_generated_columns(tokens: &mut Vec<Token>) {
    let mut index = 0;
    while index < tokens.len() {
        if keyword_of(&tokens[index]) == Some(Keyword::GENERATED) {
            if let Some(after_expr) = skip_generated_always_as_expr(tokens, index) {
                apply_stored_generated_mode(tokens, after_expr);
            }
        }
        index += 1;
    }
}

fn apply_stored_generated_mode(tokens: &mut Vec<Token>, after_expr: usize) {
    match next_non_ws(tokens, after_expr) {
        Some(idx) => match keyword_of(&tokens[idx]) {
            Some(Keyword::VIRTUAL) => set_stored(&mut tokens[idx]),
            Some(Keyword::STORED) => {}
            _ => tokens.insert(idx, stored_token()),
        },
        None => tokens.push(stored_token()),
    }
}

fn skip_generated_always_as_expr(tokens: &[Token], generated_at: usize) -> Option<usize> {
    let always_at = skip_ws(tokens, generated_at + 1);
    if keyword_of(tokens.get(always_at)?) != Some(Keyword::ALWAYS) {
        return None;
    }
    let as_at = skip_ws(tokens, always_at + 1);
    if keyword_of(tokens.get(as_at)?) != Some(Keyword::AS) {
        return None;
    }
    let open_at = skip_ws(tokens, as_at + 1);
    if !matches!(tokens.get(open_at)?, Token::LParen) {
        return None;
    }
    skip_balanced_parens(tokens, open_at)
}

fn skip_balanced_parens(tokens: &[Token], open_at: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = open_at;
    while index < tokens.len() {
        match &tokens[index] {
            Token::LParen => depth += 1,
            Token::RParen => {
                depth -= 1;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

pub(super) fn keyword_of(token: &Token) -> Option<Keyword> {
    match token {
        Token::Word(word) => Some(word.keyword),
        _ => None,
    }
}

fn set_stored(token: &mut Token) {
    if let Token::Word(word) = token {
        word.value = "STORED".to_string();
        word.keyword = Keyword::STORED;
    }
}

fn stored_token() -> Token {
    Token::Word(Word {
        value: "STORED".to_string(),
        quote_style: None,
        keyword: Keyword::STORED,
    })
}

#[cfg(test)]
mod tests;
