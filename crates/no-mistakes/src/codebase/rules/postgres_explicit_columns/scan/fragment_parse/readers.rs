use super::{clauses, is_kw, next_code, word_value, FragmentContext};
use no_mistakes_postgres_sql::idents::relation_part_key;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan};

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

pub(super) fn consume_with_head(
    tokens: &[TokenWithSpan],
    mut index: usize,
    ctx: &mut FragmentContext,
) -> usize {
    if let Some((next_i, next_tok)) = next_code(tokens, index) {
        if is_kw(next_tok, Keyword::RECURSIVE) {
            index = next_i + 1;
        }
    }
    if let Some((next_i, name)) = read_cte_name(tokens, index) {
        ctx.ctes.push(name);
        return next_i;
    }
    index
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
    let mut wrapped_only = false;
    if is_kw(tok, Keyword::ONLY) {
        let next = next_code(tokens, i + 1)?;
        i = next.0;
        tok = next.1;
        if matches!(tok.token, Token::LParen) {
            wrapped_only = true;
            let inner = next_code(tokens, i + 1)?;
            i = inner.0;
            tok = inner.1;
        }
    }
    let raw_first = word_value(tok)?;
    let mut name = relation_part_key(&raw_first);
    let mut next_i = i + 1;
    if let Some((dot_i, dot)) = next_code(tokens, next_i) {
        if matches!(dot.token, Token::Period) {
            if let Some((part_i, part)) = next_code(tokens, dot_i + 1) {
                if let Some(second) = word_value(part) {
                    name = format!("{name}.{}", relation_part_key(&second));
                    next_i = part_i + 1;
                }
            }
        }
    }
    if wrapped_only {
        if let Some((rp_i, rp)) = next_code(tokens, next_i) {
            if matches!(rp.token, Token::RParen) {
                next_i = rp_i + 1;
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
    } else if clauses::is_implicit_alias_word(alias_tok) {
        return (alias_i + 1, word_value(alias_tok));
    }
    (next_i, None)
}

pub(super) fn split_top_level_statements(tokens: &[TokenWithSpan]) -> Vec<&[TokenWithSpan]> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    for (i, tok) in tokens.iter().enumerate() {
        match tok.token {
            Token::LParen => depth += 1,
            Token::RParen => depth = depth.saturating_sub(1),
            Token::SemiColon if depth == 0 => {
                out.push(&tokens[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < tokens.len() || out.is_empty() {
        out.push(&tokens[start..]);
    }
    out
}

pub(super) fn prev_code_index(tokens: &[TokenWithSpan], before: usize) -> Option<usize> {
    (0..before)
        .rev()
        .find(|&j| !matches!(tokens[j].token, Token::Whitespace(_)))
}

pub(super) fn read_qualified_name_backwards(
    tokens: &[TokenWithSpan],
    end_idx: usize,
) -> Option<String> {
    let mut parts = vec![word_value(&tokens[end_idx])?];
    let mut cursor = end_idx;
    while let Some(dot_idx) = prev_code_index(tokens, cursor) {
        if !matches!(tokens[dot_idx].token, Token::Period) {
            break;
        }
        let Some(prev_idx) = prev_code_index(tokens, dot_idx) else {
            break;
        };
        let Some(part) = word_value(&tokens[prev_idx]) else {
            break;
        };
        parts.push(part);
        cursor = prev_idx;
    }
    parts.reverse();
    if parts.len() == 1 {
        Some(parts.pop().unwrap())
    } else {
        Some(
            parts
                .iter()
                .map(|p| relation_part_key(p))
                .collect::<Vec<_>>()
                .join("."),
        )
    }
}

pub(super) fn is_bare_star_prev(tokens: &[TokenWithSpan], pi: usize, t: &TokenWithSpan) -> bool {
    matches!(t.token, Token::Comma | Token::LParen)
        || is_kw(t, Keyword::SELECT)
        || is_kw(t, Keyword::DISTINCT)
        || is_kw(t, Keyword::ALL)
        || is_kw(t, Keyword::RETURNING)
        || (matches!(t.token, Token::RParen) && super::token_scopes::is_distinct_on_rparen(tokens, pi))
}
