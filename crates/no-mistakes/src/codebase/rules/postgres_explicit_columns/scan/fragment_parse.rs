use super::fragment_ast::{self, FragmentStar, SYNTHETIC_TARGET};
use no_mistakes_postgres_sql::parse::parse_postgres_sql;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan, Tokenizer};

mod clauses;
mod context;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod coverage_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod edge_tests;
mod readers;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
mod token_scopes;
mod tokens;
use context::extract_context_from_tokens;
use readers::{read_cte_name, read_optional_alias, read_table_and_alias, skip_paren_group};

#[derive(Debug, Default)]
pub(super) struct OuterScope {
    pub(super) relations: Vec<(String, Option<String>)>,
    pub(super) derived_aliases: Vec<String>,
}

#[derive(Debug, Default)]
pub(super) struct FragmentContext {
    pub(super) ctes: Vec<String>,
    pub(super) derived_aliases: Vec<String>,
    pub(super) has_derived_from: bool,
    pub(super) join_aliases: Vec<(String, String)>,
    pub(super) mutation_target: Option<(String, Option<String>)>,
    pub(super) from_relations: Vec<(String, Option<String>)>,
    pub(super) outer_scopes: Vec<OuterScope>,
}

pub(super) fn parse_context(prefix: &str) -> FragmentContext {
    let mut ctx = FragmentContext::default();
    let toks = tokenize(prefix);
    extract_context_from_tokens(&toks, &mut ctx);
    ctx
}

pub(super) fn extract_fragment_stars(sql: &str, context_sql: Option<&str>) -> Vec<FragmentStar> {
    if !sql.contains('*')
        && !sql
            .as_bytes()
            .windows(5)
            .any(|w| w.eq_ignore_ascii_case(b"table"))
    {
        return Vec::new();
    }
    let mut out = Vec::new();
    let open = context_sql
        .map(|prefix| clauses::open_clause_at_end(&tokenize(prefix)))
        .unwrap_or_default();
    if let Ok(statements) = parse_postgres_sql(sql) {
        if !statements.is_empty() {
            fragment_ast::collect_statements(&statements, open.in_exists, &mut out);
            return out;
        }
    }
    let toks = tokenize(sql);
    if let Some((_, token)) = next_code(&toks, 0) {
        if is_kw(token, Keyword::RETURNING) {
            let wrapped = format!("DELETE FROM {SYNTHETIC_TARGET} {sql}");
            if let Ok(statements) = parse_postgres_sql(&wrapped) {
                fragment_ast::collect_statements(&statements, false, &mut out);
                return out;
            }
        } else if matches!(token.token, Token::Comma) && (open.select || open.returning) {
            let wrapped = if open.returning {
                format!("DELETE FROM {SYNTHETIC_TARGET} RETURNING 1 {sql}")
            } else {
                format!("SELECT 1 {sql}")
            };
            if let Ok(statements) = parse_postgres_sql(&wrapped) {
                fragment_ast::collect_statements(
                    &statements,
                    !open.returning && open.in_exists,
                    &mut out,
                );
                return out;
            }
        } else if open.select && open.expects_expr {
            let wrapped = format!("SELECT {sql}");
            if let Ok(statements) = parse_postgres_sql(&wrapped) {
                fragment_ast::collect_statements(&statements, open.in_exists, &mut out);
                return out;
            }
        } else if open.returning && open.expects_expr {
            let wrapped = format!("DELETE FROM {SYNTHETIC_TARGET} RETURNING {sql}");
            if let Ok(statements) = parse_postgres_sql(&wrapped) {
                fragment_ast::collect_statements(&statements, false, &mut out);
                return out;
            }
        }
    }
    tokens::extract_stars(&toks, open, &mut out);
    out
}

fn tokenize(sql: &str) -> Vec<TokenWithSpan> {
    let mut tokens = Vec::new();
    let _ = Tokenizer::new(&PostgreSqlDialect {}, sql).tokenize_with_location_into_buf(&mut tokens);
    tokens
}

pub(super) fn next_code(
    tokens: &[TokenWithSpan],
    mut index: usize,
) -> Option<(usize, &TokenWithSpan)> {
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
        Token::Word(word) => Some(if word.quote_style.is_some() {
            word.value.clone()
        } else {
            word.value.to_ascii_lowercase()
        }),
        _ => None,
    }
}

pub(super) fn update_paren_depth(token: &TokenWithSpan, depth: &mut usize) -> bool {
    match &token.token {
        Token::LParen => {
            *depth += 1;
            true
        }
        Token::RParen => {
            *depth = depth.saturating_sub(1);
            true
        }
        _ => false,
    }
}

pub(super) fn starts_with_query(tokens: &[TokenWithSpan]) -> bool {
    tokens
        .iter()
        .find(|t| !matches!(t.token, Token::Whitespace(_)))
        .is_some_and(|t| {
            is_kw(t, Keyword::SELECT) || is_kw(t, Keyword::WITH) || is_kw(t, Keyword::VALUES)
        })
}
