use super::context::extract_context_from_tokens;
use super::{is_kw, word_value, FragmentContext};
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
            collect_ancestor_ctes(tokens, open, &mut inner.ctes);
            return Some(inner);
        }
        cursor = open;
    }
    None
}

fn collect_ancestor_ctes(tokens: &[TokenWithSpan], mut cursor: usize, ctes: &mut Vec<String>) {
    while let Some(open) = enclosing_open_paren(tokens, cursor) {
        let mut parent = FragmentContext::default();
        extract_context_from_tokens(&tokens[open + 1..cursor], &mut parent);
        ctes.extend(parent.ctes);
        cursor = open;
    }
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

fn starts_with_query(tokens: &[TokenWithSpan]) -> bool {
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
    Some(parts.join("."))
}
