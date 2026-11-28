use super::context::extract_context_from_tokens;
use super::{is_kw, word_value, FragmentContext};
use no_mistakes_postgres_sql::idents::{
    relation_part_key, relation_suffix_key, relation_suffix_name,
};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn scope_context(
    tokens: &[TokenWithSpan],
    star_index: usize,
    outer: &FragmentContext,
) -> Option<FragmentContext> {
    let mut cursor = star_index;
    while let Some(open) = enclosing_open_paren(tokens, cursor) {
        let close = matching_close_paren(tokens, open);
        let slice = &tokens[open + 1..close];
        let mut inner = FragmentContext {
            ctes: outer.ctes.clone(),
            ..Default::default()
        };
        extract_context_from_tokens(slice, &mut inner);
        if !inner.from_relations.is_empty()
            || inner.mutation_target.is_some()
            || starts_with_query(slice)
        {
            collect_ancestors(tokens, open, outer, &mut inner);
            return Some(inner);
        }
        cursor = open;
    }
    None
}

fn collect_ancestors(
    tokens: &[TokenWithSpan],
    mut cursor: usize,
    outer: &FragmentContext,
    inner: &mut FragmentContext,
) {
    while let Some(open) = enclosing_open_paren(tokens, cursor) {
        let close = matching_close_paren(tokens, open);
        let mut parent_ctes = FragmentContext::default();
        extract_context_from_tokens(&tokens[open + 1..cursor], &mut parent_ctes);
        inner.ctes.extend(parent_ctes.ctes);
        let mut parent_full = FragmentContext::default();
        extract_context_from_tokens(&tokens[open + 1..close], &mut parent_full);
        inner.outer_relations.extend(parent_full.from_relations);
        cursor = open;
    }
    inner.outer_relations.extend(outer.from_relations.iter().cloned());
}

pub(super) fn is_distinct_on_rparen(tokens: &[TokenWithSpan], rparen_index: usize) -> bool {
    let Some(open) = enclosing_open_paren(tokens, rparen_index) else {
        return false;
    };
    let Some(on_idx) = prev_code_index(tokens, open) else {
        return false;
    };
    if !is_kw(&tokens[on_idx], Keyword::ON) {
        return false;
    }
    prev_code_index(tokens, on_idx).is_some_and(|idx| is_kw(&tokens[idx], Keyword::DISTINCT))
}

fn prev_code_index(tokens: &[TokenWithSpan], before: usize) -> Option<usize> {
    (0..before)
        .rev()
        .find(|&j| !matches!(tokens[j].token, Token::Whitespace(_)))
}

pub(super) fn starts_with_query(tokens: &[TokenWithSpan]) -> bool {
    tokens
        .iter()
        .find(|t| !matches!(t.token, Token::Whitespace(_)))
        .is_some_and(|t| is_kw(t, Keyword::SELECT) || is_kw(t, Keyword::WITH))
}

fn enclosing_open_paren(tokens: &[TokenWithSpan], star_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    for idx in (0..star_index).rev() {
        match tokens[idx].token {
            Token::RParen => depth += 1,
            Token::LParen if depth == 0 => return Some(idx),
            Token::LParen => depth -= 1,
            _ => {}
        }
    }
    None
}

fn matching_close_paren(tokens: &[TokenWithSpan], open: usize) -> usize {
    let mut depth = 0usize;
    for (idx, tok) in tokens.iter().enumerate().skip(open + 1) {
        match tok.token {
            Token::LParen => depth += 1,
            Token::RParen if depth == 0 => return idx,
            Token::RParen => depth -= 1,
            _ => {}
        }
    }
    tokens.len()
}

pub(super) fn enclosing_function(tokens: &[TokenWithSpan], star_index: usize) -> Option<String> {
    let mut cursor = star_index;
    while let Some(open) = enclosing_open_paren(tokens, cursor) {
        let prev = (0..open)
            .rev()
            .find(|&j| !matches!(tokens[j].token, Token::Whitespace(_)))?;
        match &tokens[prev].token {
            Token::LParen => cursor = open,
            Token::Word(word)
                if !matches!(
                    word.keyword,
                    sqlparser::keywords::Keyword::SELECT
                        | sqlparser::keywords::Keyword::WHERE
                        | sqlparser::keywords::Keyword::AND
                        | sqlparser::keywords::Keyword::OR
                        | sqlparser::keywords::Keyword::IN
                        | sqlparser::keywords::Keyword::EXISTS
                        | sqlparser::keywords::Keyword::ON
                        | sqlparser::keywords::Keyword::THEN
                        | sqlparser::keywords::Keyword::ELSE
                        | sqlparser::keywords::Keyword::WHEN
                        | sqlparser::keywords::Keyword::CASE
                        | sqlparser::keywords::Keyword::NOT
                        | sqlparser::keywords::Keyword::RETURNING
                ) =>
            {
                return read_qualified_name_backwards(tokens, prev);
            }
            _ => return None,
        }
    }
    None
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

pub(super) fn resolve_qualified_relation(ctx: &FragmentContext, q: &str) -> Option<String> {
    let find_in = |rels: &[(String, Option<String>)]| {
        rels.iter()
            .find(|(_, a)| a.as_deref() == Some(q))
            .or_else(|| {
                rels.iter().find(|(t, a)| {
                    a.is_none()
                        && (t == q || relation_suffix_key(t) == q || relation_suffix_name(t) == q)
                })
            })
            .map(|(t, _)| t.clone())
    };
    let local: Vec<_> = ctx
        .from_relations
        .iter()
        .chain(ctx.mutation_target.iter())
        .cloned()
        .collect();
    if let Some(found) = find_in(&local) {
        return Some(found);
    }
    if ctx.derived_aliases.iter().any(|a| a == q) {
        return None;
    }
    find_in(&ctx.outer_relations)
}
