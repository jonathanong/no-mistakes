//! Split rewritten SQL tokens into nonempty chunks and navigate whitespace.
use sqlparser::tokenizer::Token;

pub(super) fn split_statement_tokens(tokens: Vec<Token>) -> Vec<Vec<Token>> {
    let mut statements = Vec::new();
    let mut current = Vec::new();
    for token in tokens {
        if matches!(token, Token::SemiColon) {
            if has_non_ws(&current) {
                statements.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
        } else {
            current.push(token);
        }
    }
    if has_non_ws(&current) {
        statements.push(current);
    }
    statements
}

pub(super) fn has_non_ws(tokens: &[Token]) -> bool {
    tokens
        .iter()
        .any(|token| !matches!(token, Token::Whitespace(_)))
}

pub(in super::super) fn skip_ws(tokens: &[Token], mut index: usize) -> usize {
    while index < tokens.len() && matches!(tokens[index], Token::Whitespace(_)) {
        index += 1;
    }
    index
}

pub(in super::super) fn next_non_ws(tokens: &[Token], start: usize) -> Option<usize> {
    let index = skip_ws(tokens, start);
    (index < tokens.len()).then_some(index)
}
