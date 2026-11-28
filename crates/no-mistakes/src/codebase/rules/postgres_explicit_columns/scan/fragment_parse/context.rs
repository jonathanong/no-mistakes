use super::{is_kw, next_code, word_value, FragmentContext};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn extract_context_from_tokens(tokens: &[TokenWithSpan], ctx: &mut FragmentContext) {
    let mut index = 0;
    let mut depth = 0usize;
    let mut in_with = false;
    while let Some((i, token)) = next_code(tokens, index) {
        index = i + 1;
        match &token.token {
            Token::LParen => {
                depth += 1;
                continue;
            }
            Token::RParen => {
                depth = depth.saturating_sub(1);
                continue;
            }
            _ => {}
        }
        if depth > 0 {
            continue;
        }
        if is_kw(token, Keyword::WITH) {
            in_with = true;
            if let Some((next_i, next_tok)) = next_code(tokens, index) {
                if is_kw(next_tok, Keyword::RECURSIVE) {
                    index = next_i + 1;
                }
            }
            if let Some((next_i, name)) = read_cte_name(tokens, index) {
                ctx.ctes.push(name);
                index = next_i;
            }
            continue;
        }
        if in_with && matches!(token.token, Token::Comma) {
            if let Some((next_i, name)) = read_cte_name(tokens, index) {
                ctx.ctes.push(name);
                index = next_i;
                continue;
            }
        }
        if is_kw(token, Keyword::UPDATE) {
            in_with = false;
            if let Some((next_i, rel)) = read_table_and_alias(tokens, index) {
                ctx.mutation_target = Some(rel);
                index = next_i;
            }
        } else if is_kw(token, Keyword::INTO) || is_kw(token, Keyword::FROM) {
            let is_into = is_kw(token, Keyword::INTO);
            in_with = false;
            if let Some((next_i, rel)) = read_table_and_alias(tokens, index) {
                if is_into {
                    ctx.mutation_target = Some(rel);
                } else {
                    if ctx.mutation_target.is_none()
                        && tokens[..i].iter().any(|t| is_kw(t, Keyword::DELETE))
                    {
                        ctx.mutation_target = Some(rel.clone());
                    }
                    ctx.from_relations.push(rel);
                }
                index = next_i;
            }
        } else if is_kw(token, Keyword::JOIN) {
            if let Some((next_i, rel)) = read_table_and_alias(tokens, index) {
                ctx.from_relations.push(rel);
                index = next_i;
            }
        }
    }
}

fn read_cte_name(tokens: &[TokenWithSpan], index: usize) -> Option<(usize, String)> {
    let (i, name_tok) = next_code(tokens, index)?;
    let name = word_value(name_tok)?;
    let (mut cursor, mut next) = next_code(tokens, i + 1)?;
    if matches!(next.token, Token::LParen) {
        let mut depth = 1usize;
        cursor += 1;
        while let Some(tok) = tokens.get(cursor) {
            match tok.token {
                Token::LParen => depth += 1,
                Token::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        cursor += 1;
                        break;
                    }
                }
                _ => {}
            }
            cursor += 1;
        }
        let (as_idx, as_tok) = next_code(tokens, cursor)?;
        cursor = as_idx;
        next = as_tok;
    }
    is_kw(next, Keyword::AS).then_some((cursor + 1, name))
}

fn read_table_and_alias(
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
    let Some((alias_i, alias_tok)) = next_code(tokens, next_i) else {
        return Some((next_i, (name, None)));
    };
    if is_kw(alias_tok, Keyword::AS) {
        if let Some((id_i, id_tok)) = next_code(tokens, alias_i + 1) {
            if let Some(alias) = word_value(id_tok) {
                return Some((id_i + 1, (name, Some(alias))));
            }
        }
    } else if let Token::Word(word) = &alias_tok.token {
        if word.keyword == Keyword::NoKeyword {
            return Some((alias_i + 1, (name, Some(word.value.to_ascii_lowercase()))));
        }
    }
    Some((next_i, (name, None)))
}
