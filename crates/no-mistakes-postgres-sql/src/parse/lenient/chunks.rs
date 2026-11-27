//! Split rewritten SQL tokens into nonempty chunks and navigate whitespace.
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn located_chunks(original: &[TokenWithSpan]) -> impl Iterator<Item = &[TokenWithSpan]> {
    let mut boundary = StatementBoundary::default();
    original
        .split(move |token| boundary.terminator(&token.token))
        .filter(|chunk| {
            chunk
                .iter()
                .any(|token| !matches!(token.token, Token::Whitespace(_)))
        })
}

pub(super) fn split_statement_tokens(tokens: Vec<Token>) -> Vec<Vec<Token>> {
    let mut statements = Vec::new();
    let mut current = Vec::new();
    let mut boundary = StatementBoundary::default();
    for token in tokens {
        if boundary.terminator(&token) {
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

/// Only a CREATE RULE action list owns unquoted, parenthesized semicolons.
/// Ordinary malformed parentheses must not swallow later migration statements.
#[derive(Default)]
pub(super) struct StatementBoundary {
    prefix: u8,
    rule: bool,
    depth: usize,
}

impl StatementBoundary {
    pub(super) fn terminator(&mut self, token: &Token) -> bool {
        if matches!(token, Token::SemiColon) && (!self.rule || self.depth == 0) {
            *self = Self::default();
            return true;
        }
        if matches!(token, Token::Whitespace(_)) {
            return false;
        }
        let word = |expected: &str| {
            matches!(token, Token::Word(word)
            if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
        };
        self.prefix = match self.prefix {
            0 if word("CREATE") => 1,
            1 if word("OR") => 2,
            2 if word("REPLACE") => 3,
            1 | 3 if word("RULE") => {
                self.rule = true;
                4
            }
            _ => 4,
        };
        if self.rule {
            match token {
                Token::LParen => self.depth += 1,
                Token::RParen => self.depth = self.depth.saturating_sub(1),
                _ => {}
            }
        }
        false
    }
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
