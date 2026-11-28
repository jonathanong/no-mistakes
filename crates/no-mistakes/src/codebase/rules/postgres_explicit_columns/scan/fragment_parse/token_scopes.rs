use super::context::extract_context_from_tokens;
use super::{word_value, FragmentContext};
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn scope_context(
    tokens: &[TokenWithSpan],
    star_index: usize,
    outer: &FragmentContext,
) -> Option<FragmentContext> {
    let open = enclosing_open_paren(tokens, star_index)?;
    let close = matching_close_paren(tokens, open);
    let mut inner = FragmentContext {
        ctes: outer.ctes.clone(),
        ..Default::default()
    };
    extract_context_from_tokens(&tokens[open + 1..close], &mut inner);
    (!inner.from_relations.is_empty() || inner.mutation_target.is_some()).then_some(inner)
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
    let open = enclosing_open_paren(tokens, star_index)?;
    for j in (0..open).rev() {
        if matches!(tokens[j].token, Token::Whitespace(_)) {
            continue;
        }
        return word_value(&tokens[j]);
    }
    None
}
