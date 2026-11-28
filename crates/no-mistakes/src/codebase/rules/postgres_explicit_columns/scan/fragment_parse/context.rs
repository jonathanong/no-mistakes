use super::{
    is_kw, next_code, read_cte_name, read_optional_alias, read_table_and_alias, skip_paren_group,
    FragmentContext,
};
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
        if let Some(next_i) = consume_from_factor(tokens, index, false, i, ctx) {
            return next_i;
        }
    }
    consume_clause_relation(tokens, i, token, index, in_with, in_from, ctx)
}

fn consume_from_factor(
    tokens: &[TokenWithSpan],
    index: usize,
    is_into: bool,
    kw_i: usize,
    ctx: &mut FragmentContext,
) -> Option<usize> {
    if !is_into {
        if let Some((paren_i, tok)) = next_code(tokens, index) {
            if matches!(tok.token, Token::LParen) {
                let after = skip_paren_group(tokens, paren_i);
                let (end, alias) = read_optional_alias(tokens, after);
                let inner = &tokens[paren_i + 1..after.saturating_sub(1)];
                if super::token_scopes::starts_with_query(inner) {
                    if let Some(name) = alias {
                        ctx.derived_aliases.push(name);
                    }
                } else {
                    extract_parenthesized_join(inner, alias.as_deref(), ctx);
                }
                return Some(end);
            }
        }
    }
    let (next_i, rel) = read_table_and_alias(tokens, index)?;
    record_from_or_into(tokens, kw_i, is_into, rel, ctx);
    Some(next_i)
}

fn extract_parenthesized_join(
    inner: &[TokenWithSpan],
    outer_alias: Option<&str>,
    ctx: &mut FragmentContext,
) {
    let mut sub = FragmentContext::default();
    if let Some(next_i) = consume_from_factor(inner, 0, false, 0, &mut sub) {
        extract_context_from_tokens(&inner[next_i..], &mut sub);
    }
    if let Some(alias) = outer_alias {
        for (table, _) in &sub.from_relations {
            ctx.join_aliases.push((alias.to_string(), table.clone()));
        }
    }
    ctx.from_relations.extend(sub.from_relations);
    ctx.join_aliases.extend(sub.join_aliases);
    ctx.derived_aliases.extend(sub.derived_aliases);
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
        let is_do = (0..i)
            .rev()
            .find(|&j| !matches!(tokens[j].token, Token::Whitespace(_)))
            .is_some_and(|j| is_kw(&tokens[j], Keyword::DO));
        if !is_do {
            if let Some((next_i, rel)) = read_table_and_alias(tokens, index) {
                ctx.mutation_target = Some(rel);
                return next_i;
            }
        }
    } else if is_kw(token, Keyword::INTO) || is_kw(token, Keyword::FROM) || is_kw(token, Keyword::USING) {
        let is_into = is_kw(token, Keyword::INTO);
        *in_with = false;
        *in_from = !is_into;
        if let Some(next_i) = consume_from_factor(tokens, index, is_into, i, ctx) {
            return next_i;
        }
    } else if is_kw(token, Keyword::JOIN) {
        *in_from = true;
        if let Some(next_i) = consume_from_factor(tokens, index, false, i, ctx) {
            return next_i;
        }
    } else if super::clauses::ends_from_list(token) {
        if is_kw(token, Keyword::SELECT) || is_kw(token, Keyword::DELETE) {
            *in_with = false;
        }
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
