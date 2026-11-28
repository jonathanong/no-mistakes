use super::fragment_ast::{self, FragmentStar, SYNTHETIC_TARGET};
use no_mistakes_postgres_sql::parse::parse_postgres_sql;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan, Tokenizer};

mod context;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
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
            fragment_ast::collect_statements(&statements, &mut out);
            return out;
        }
    }
    let (ctx_select, ctx_returning) = context_sql
        .map(|prefix| open_clause_at_end(&tokenize(prefix)))
        .unwrap_or((false, false));
    let toks = tokenize(sql);
    if let Some((_, token)) = next_code(&toks, 0) {
        if is_kw(token, Keyword::RETURNING) {
            let wrapped = format!("DELETE FROM {SYNTHETIC_TARGET} {sql}");
            if let Ok(statements) = parse_postgres_sql(&wrapped) {
                fragment_ast::collect_statements(&statements, &mut out);
                return out;
            }
        } else if matches!(token.token, Token::Comma) {
            let wrapped = if ctx_returning {
                format!("DELETE FROM {SYNTHETIC_TARGET} RETURNING 1 {sql}")
            } else {
                format!("SELECT 1 {sql}")
            };
            if let Ok(statements) = parse_postgres_sql(&wrapped) {
                fragment_ast::collect_statements(&statements, &mut out);
                return out;
            }
        } else if ctx_select {
            let wrapped = format!("SELECT {sql}");
            if let Ok(statements) = parse_postgres_sql(&wrapped) {
                fragment_ast::collect_statements(&statements, &mut out);
                return out;
            }
        } else if ctx_returning {
            let wrapped = format!("DELETE FROM {SYNTHETIC_TARGET} RETURNING {sql}");
            if let Ok(statements) = parse_postgres_sql(&wrapped) {
                fragment_ast::collect_statements(&statements, &mut out);
                return out;
            }
        }
    }
    tokens::extract_stars(&toks, ctx_select, ctx_returning, &mut out);
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

fn open_clause_at_end(tokens: &[TokenWithSpan]) -> (bool, bool) {
    let mut depth = 0usize;
    let mut select = false;
    let mut returning = false;
    for token in tokens {
        if matches!(token.token, Token::Whitespace(_)) {
            continue;
        }
        match &token.token {
            Token::LParen => depth += 1,
            Token::RParen => depth = depth.saturating_sub(1),
            _ if depth > 0 => {}
            _ if is_kw(token, Keyword::SELECT) => {
                select = true;
                returning = false;
            }
            _ if is_kw(token, Keyword::RETURNING) => {
                returning = true;
                select = false;
            }
            _ if is_kw(token, Keyword::FROM)
                || is_kw(token, Keyword::WHERE)
                || is_kw(token, Keyword::INTO)
                || is_kw(token, Keyword::SET)
                || is_kw(token, Keyword::VALUES)
                || is_kw(token, Keyword::GROUP)
                || is_kw(token, Keyword::ORDER)
                || is_kw(token, Keyword::LIMIT)
                || is_kw(token, Keyword::HAVING)
                || is_kw(token, Keyword::UNION)
                || is_kw(token, Keyword::INTERSECT)
                || is_kw(token, Keyword::EXCEPT) =>
            {
                select = false;
                returning = false;
            }
            _ => {}
        }
    }
    (select, returning)
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

fn skip_paren_group(tokens: &[TokenWithSpan], start: usize) -> usize {
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
