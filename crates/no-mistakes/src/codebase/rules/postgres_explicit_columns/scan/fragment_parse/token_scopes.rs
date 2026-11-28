use super::context::extract_context_from_tokens;
use super::readers::{prev_code_index, read_qualified_name_backwards};
use super::{is_kw, FragmentContext, OuterScope};
use no_mistakes_postgres_sql::idents::{relation_suffix_key, relation_suffix_name};
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
            || super::starts_with_query(slice)
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
        inner.outer_scopes.push(OuterScope {
            relations: parent_full.from_relations,
            derived_aliases: parent_full.derived_aliases,
        });
        cursor = open;
    }
    inner.outer_scopes.push(OuterScope {
        relations: outer.from_relations.clone(),
        derived_aliases: outer.derived_aliases.clone(),
    });
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
        if super::starts_with_query(&tokens[open + 1..star_index]) {
            return None;
        }
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

pub(super) fn resolve_qualified_relation(
    ctx: &FragmentContext,
    q: &str,
    returning: bool,
) -> Option<(String, bool)> {
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
    if returning {
        if let Some(target) = &ctx.mutation_target {
            if let Some(found) = find_in(std::slice::from_ref(target)) {
                return Some((found, true));
            }
        }
    }
    if let Some(found) = find_in(&ctx.from_relations) {
        return Some((found, false));
    }
    if !returning {
        if let Some(target) = &ctx.mutation_target {
            if let Some(found) = find_in(std::slice::from_ref(target)) {
                return Some((found, false));
            }
        }
    }
    if ctx.derived_aliases.iter().any(|a| a == q) {
        return None;
    }
    for level in &ctx.outer_scopes {
        if let Some(found) = find_in(&level.relations) {
            return Some((found, false));
        }
        if level.derived_aliases.iter().any(|a| a == q) {
            return None;
        }
    }
    None
}

pub(super) fn is_known_cte_or_derived(ctx: &FragmentContext, q: &str) -> bool {
    ctx.ctes.iter().any(|c| c == q)
        || ctx.derived_aliases.iter().any(|a| a == q)
        || ctx
            .outer_scopes
            .iter()
            .any(|s| s.derived_aliases.iter().any(|a| a == q))
}

pub(super) fn is_cte_relation(ctx: &FragmentContext, rel: &str) -> bool {
    relation_suffix_key(rel) == rel
        && ctx
            .ctes
            .iter()
            .any(|c| c == rel || c == &relation_suffix_name(rel))
}
