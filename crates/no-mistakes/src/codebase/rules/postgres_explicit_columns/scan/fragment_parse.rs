use super::fragment_ast::{self, FragmentStar, SYNTHETIC_TARGET};
use no_mistakes_postgres_sql::parse::parse_postgres_sql;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan, Tokenizer};

mod context;
mod tokens;
use context::extract_context_from_tokens;

#[derive(Debug, Default)]
pub(super) struct FragmentContext {
    pub(super) ctes: Vec<String>,
    pub(super) mutation_target: Option<(String, Option<String>)>,
    pub(super) from_relations: Vec<(String, Option<String>)>,
}

pub(super) fn parse_context(prefix: &str) -> FragmentContext {
    let mut ctx = FragmentContext::default();
    let toks = tokenize(prefix);
    extract_context_from_tokens(&toks, &mut ctx);
    ctx
}

pub(super) fn extract_fragment_stars(sql: &str) -> Vec<FragmentStar> {
    if !sql.contains('*') {
        return Vec::new();
    }
    let mut out = Vec::new();
    if let Ok(statements) = parse_postgres_sql(sql) {
        if !statements.is_empty() {
            fragment_ast::collect_statements(&statements, &mut out);
            return out;
        }
    }
    let toks = tokenize(sql);
    if let Some((_, token)) = next_code(&toks, 0) {
        if is_kw(token, Keyword::RETURNING) {
            let wrapped = format!("DELETE FROM {SYNTHETIC_TARGET} {sql}");
            if let Ok(statements) = parse_postgres_sql(&wrapped) {
                fragment_ast::collect_statements(&statements, &mut out);
                return out;
            }
        } else if matches!(token.token, Token::Comma) {
            let wrapped = format!("SELECT 1 {sql}");
            if let Ok(statements) = parse_postgres_sql(&wrapped) {
                fragment_ast::collect_statements(&statements, &mut out);
                return out;
            }
        }
    }
    tokens::extract_stars(&toks, &mut out);
    out
}

fn tokenize(sql: &str) -> Vec<TokenWithSpan> {
    Tokenizer::new(&PostgreSqlDialect {}, sql)
        .tokenize_with_location()
        .unwrap_or_default()
}

pub(super) fn next_code(tokens: &[TokenWithSpan], mut index: usize) -> Option<(usize, &TokenWithSpan)> {
    while let Some(token) = tokens.get(index) {
        if !matches!(token.token, Token::Whitespace(_)) {
            return Some((index, token));
        }
        index += 1;
    }
    None
}

pub(super) fn is_kw(token: &TokenWithSpan, keyword: Keyword) -> bool {
    matches!(&token.token, Token::Word(word) if word.keyword == keyword)
}

pub(super) fn word_value(token: &TokenWithSpan) -> Option<String> {
    match &token.token {
        Token::Word(word) => Some(word.value.to_ascii_lowercase()),
        _ => None,
    }
}
