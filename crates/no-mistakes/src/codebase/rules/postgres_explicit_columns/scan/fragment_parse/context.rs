use super::{is_kw, next_code, read_cte_name, word_value, FragmentContext};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn extract_context_from_tokens(tokens: &[TokenWithSpan], ctx: &mut FragmentContext) {
    let mut index = 0;
    let mut depth = 0usize;
    let mut in_with = false;
    let mut in_from = false;
    while let Some((i, token)) = next_code(tokens, index) {
        index = i + 1;
        if update_paren_depth(token, &mut depth) || depth > 0 {
            continue;
        }
        index = step_top_level(tokens, i, token, index, &mut in_with, &mut in_from, ctx);
    }
}

fn update_paren_depth(token: &TokenWithSpan, depth: &mut usize) -> bool {
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

fn step_top_level(
    tokens: &[TokenWithSpan],
    i: usize,
    token: &TokenWithSpan,
    index: usize,
    in_with: &mut bool,
    in_from: &mut bool,
    ctx: &mut FragmentContext,
) -> usize {
    if is_kw(token, Keyword::WITH) {
        *in_with = true;
        *in_from = false;
        return consume_with_head(tokens, index, ctx);
    }
    if *in_with && matches!(token.token, Token::Comma) {
        if let Some((next_i, name)) = read_cte_name(tokens, index) {
            ctx.ctes.push(name);
            return next_i;
        }
    }
    if *in_from && matches!(token.token, Token::Comma) {
        if let Some((next_i, rel)) = read_table_and_alias(tokens, index) {
            ctx.from_relations.push(rel);
            return next_i;
        }
    }
    consume_clause_relation(tokens, i, token, index, in_with, in_from, ctx)
}

fn consume_with_head(tokens: &[TokenWithSpan], mut index: usize, ctx: &mut FragmentContext) -> usize {
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

fn consume_clause_relation(
    tokens: &[TokenWithSpan],
    i: usize,
    token: &TokenWithSpan,
    index: usize,
    in_with: &mut bool,
    in_from: &mut bool,
    ctx: &mut FragmentContext,
) -> usize {
    if is_kw(token, Keyword::UPDATE) {
        *in_with = false;
        *in_from = false;
        if let Some((next_i, rel)) = read_table_and_alias(tokens, index) {
            ctx.mutation_target = Some(rel);
            return next_i;
        }
    } else if is_kw(token, Keyword::INTO) || is_kw(token, Keyword::FROM) || is_kw(token, Keyword::USING) {
        let is_into = is_kw(token, Keyword::INTO);
        *in_with = false;
        *in_from = !is_into;
        if let Some((next_i, rel)) = read_table_and_alias(tokens, index) {
            record_from_or_into(tokens, i, is_into, rel, ctx);
            return next_i;
        }
    } else if is_kw(token, Keyword::JOIN) {
        *in_from = true;
        if let Some((next_i, rel)) = read_table_and_alias(tokens, index) {
            ctx.from_relations.push(rel);
            return next_i;
        }
    } else if ends_from_list(token) {
        *in_from = false;
    }
    index
}

fn record_from_or_into(
    tokens: &[TokenWithSpan],
    i: usize,
    is_into: bool,
    rel: (String, Option<String>),
    ctx: &mut FragmentContext,
) {
    if is_into {
        ctx.mutation_target = Some(rel);
    } else {
        if ctx.mutation_target.is_none() && tokens[..i].iter().any(|t| is_kw(t, Keyword::DELETE)) {
            ctx.mutation_target = Some(rel.clone());
        }
        ctx.from_relations.push(rel);
    }
}

fn ends_from_list(token: &TokenWithSpan) -> bool {
    matches!(
        &token.token,
        Token::Word(w) if matches!(
            w.keyword,
            Keyword::WHERE
                | Keyword::GROUP
                | Keyword::HAVING
                | Keyword::ORDER
                | Keyword::LIMIT
                | Keyword::OFFSET
                | Keyword::FETCH
                | Keyword::UNION
                | Keyword::INTERSECT
                | Keyword::EXCEPT
                | Keyword::RETURNING
                | Keyword::SET
                | Keyword::VALUES
                | Keyword::SELECT
                | Keyword::DELETE
                | Keyword::WINDOW
        )
    )
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
