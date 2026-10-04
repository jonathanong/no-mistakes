use super::standalone_table::{advance_previous, is_table, preceded_by_set_op};
use sqlparser::tokenizer::{Token, TokenWithSpan};

/// sqlparser consumes three tokens for an unqualified TABLE arm, including the
/// delimiter and the next statement's first token. Same-span delimiter padding
/// leaves the original delimiter available without rewriting the TABLE AST.
pub(super) fn normalize(tokens: &mut Vec<TokenWithSpan>) {
    let mut delimiters = Vec::new();
    let (mut previous, mut before_previous) = (None, None);
    for (at, token) in tokens.iter().enumerate() {
        if is_table(&token.token) && preceded_by_set_op(previous, before_previous) {
            if let Some(delimiter) = delimiter_after(tokens, at) {
                delimiters.push(delimiter);
            }
        }
        advance_previous(&token.token, &mut previous, &mut before_previous);
    }
    if delimiters.is_empty() {
        return;
    }
    let mut delimiters = delimiters.into_iter().peekable();
    let mut result = Vec::with_capacity(tokens.len());
    for (at, token) in tokens.drain(..).enumerate() {
        if delimiters.peek() == Some(&at) {
            delimiters.next();
            result.extend([token.clone(), token.clone()]);
        }
        result.push(token);
    }
    *tokens = result;
}

fn delimiter_after(tokens: &[TokenWithSpan], at: usize) -> Option<usize> {
    let mut after = tokens[at + 1..]
        .iter()
        .enumerate()
        .filter(|(_, token)| !matches!(token.token, Token::Whitespace(_)));
    if !matches!(after.next(), Some((_, token)) if matches!(token.token, Token::Word(_))) {
        return None;
    }
    let (delimiter, token) = after.next()?;
    if token.token != Token::SemiColon {
        return None;
    }
    after
        .any(|(_, token)| token.token != Token::SemiColon)
        .then_some(at + delimiter + 1)
}

#[cfg(test)]
mod tests;
