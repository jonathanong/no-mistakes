use super::fragment_ast::{self, FragmentStar, SYNTHETIC_TARGET};
use no_mistakes_postgres_sql::parse::parse_postgres_sql;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan, Tokenizer};

mod clauses;
mod context;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
mod token_scopes;
mod tokens;
use context::extract_context_from_tokens;

#[derive(Debug, Default)]
pub(super) struct FragmentContext {
    pub(super) ctes: Vec<String>,
    pub(super) derived_aliases: Vec<String>,
    pub(super) join_aliases: Vec<(String, String)>,
    pub(super) mutation_target: Option<(String, Option<String>)>,
    pub(super) from_relations: Vec<(String, Option<String>)>,
    pub(super) outer_relations: Vec<(String, Option<String>)>,
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
    if let Ok(statements) = parse_postgres_sql(sql) {
        if !statements.is_empty() {
            fragment_ast::collect_statements(&statements, false, &mut out);
            return out;
        }
    }
    let (ctx_select, ctx_returning, ctx_exists) = context_sql
        .map(|prefix| clauses::open_clause_at_end(&tokenize(prefix)))
        .unwrap_or((false, false, false));
    let toks = tokenize(sql);
    if let Some((_, token)) = next_code(&toks, 0) {
        if is_kw(token, Keyword::RETURNING) {
            let wrapped = format!("DELETE FROM {SYNTHETIC_TARGET} {sql}");
            if let Ok(statements) = parse_postgres_sql(&wrapped) {
                fragment_ast::collect_statements(&statements, false, &mut out);
                return out;
            }
        } else if matches!(token.token, Token::Comma) && (ctx_select || ctx_returning) {
            let wrapped = if ctx_returning {
                format!("DELETE FROM {SYNTHETIC_TARGET} RETURNING 1 {sql}")
            } else {
                format!("SELECT 1 {sql}")
            };
            if let Ok(statements) = parse_postgres_sql(&wrapped) {
                fragment_ast::collect_statements(&statements, !ctx_returning && ctx_exists, &mut out);
                return out;
            }
        } else if ctx_select {
            let wrapped = format!("SELECT {sql}");
            if let Ok(statements) = parse_postgres_sql(&wrapped) {
                fragment_ast::collect_statements(&statements, ctx_exists, &mut out);
                return out;
            }
        } else if ctx_returning {
            let wrapped = format!("DELETE FROM {SYNTHETIC_TARGET} RETURNING {sql}");
            if let Ok(statements) = parse_postgres_sql(&wrapped) {
                fragment_ast::collect_statements(&statements, false, &mut out);
                return out;
            }
        }
    }
    tokens::extract_stars(&toks, ctx_select, ctx_returning, ctx_exists, &mut out);
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
        Token::Word(word) => Some(if word.quote_style.is_some() {
            word.value.clone()
        } else {
            word.value.to_ascii_lowercase()
        }),
        _ => None,
    }
}

pub(super) fn read_cte_name(tokens: &[TokenWithSpan], index: usize) -> Option<(usize, String)> {
    let (i, name_tok) = next_code(tokens, index)?;
    let name = word_value(name_tok)?;
    let (cursor, next) = next_code(tokens, i + 1)?;
    let (as_idx, as_tok) = if matches!(next.token, Token::LParen) {
        next_code(tokens, skip_paren_group(tokens, cursor))?
    } else {
        (cursor, next)
    };
    is_kw(as_tok, Keyword::AS).then_some((as_idx + 1, name))
}

pub(super) fn skip_paren_group(tokens: &[TokenWithSpan], start: usize) -> usize {
    let mut depth = 1usize;
    let mut cursor = start + 1;
    while let Some(tok) = tokens.get(cursor) {
        match tok.token {
            Token::LParen => depth += 1,
            Token::RParen => {
                depth -= 1;
                if depth == 0 {
                    return cursor + 1;
                }
            }
            _ => {}
        }
        cursor += 1;
    }
    cursor
}

pub(super) fn read_table_and_alias(
    tokens: &[TokenWithSpan],
    index: usize,
) -> Option<(usize, (String, Option<String>))> {
    let (mut i, mut tok) = next_code(tokens, index)?;
    if is_kw(tok, Keyword::ONLY) {
        let next = next_code(tokens, i + 1)?;
        i = next.0;
        tok = next.1;
    }
    let mut name = word_value(tok)?;
    let mut next_i = i + 1;
    if let Some((dot_i, dot)) = next_code(tokens, next_i) {
        if matches!(dot.token, Token::Period) {
            if let Some((part_i, part)) = next_code(tokens, dot_i + 1) {
                if let Some(second) = word_value(part) {
                    name = format!("{name}.{second}");
                    next_i = part_i + 1;
                }
            }
        }
    }
    let (end, alias) = read_optional_alias(tokens, next_i);
    Some((end, (name, alias)))
}

pub(super) fn read_optional_alias(
    tokens: &[TokenWithSpan],
    next_i: usize,
) -> (usize, Option<String>) {
    let Some((alias_i, alias_tok)) = next_code(tokens, next_i) else {
        return (next_i, None);
    };
    if is_kw(alias_tok, Keyword::AS) {
        if let Some((id_i, id_tok)) = next_code(tokens, alias_i + 1) {
            if let Some(alias) = word_value(id_tok) {
                return (id_i + 1, Some(alias));
            }
        }
    } else if let Token::Word(word) = &alias_tok.token {
        if word.keyword == Keyword::NoKeyword {
            return (alias_i + 1, word_value(alias_tok));
        }
    }
    (next_i, None)
}
