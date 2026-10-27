//! Repair sqlparser's decimal/range ambiguity only in the procedural classifier.
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn marker(token: &Token) -> bool {
    matches!(token, Token::CustomBinaryOperator(value) if value == "..")
}

pub(super) fn normalize(tokens: &[TokenWithSpan]) -> Vec<Token> {
    let mut out = Vec::with_capacity(tokens.len());
    let mut at = 0;
    while at < tokens.len() {
        let token = &tokens[at].token;
        // Keep whitespace/comments in this pass: `1. .bound` is not `1..bound`.
        let left = match token {
            Token::Number(value, false) => value.strip_suffix('.').map(Some),
            Token::Period => Some(None),
            _ => None,
        };
        let right = tokens.get(at + 1).and_then(|next| match &next.token {
            Token::Number(value, false) => value.strip_prefix('.').map(Some),
            Token::Period => Some(None),
            _ => None,
        });
        if let (Some(left), Some(right)) = (left, right) {
            if let Some(left) = left {
                out.push(Token::Number(left.into(), false));
            }
            out.push(Token::CustomBinaryOperator("..".into()));
            at += 2;
            if let Some(right) = right {
                // After `1.5..2.5`, sqlparser emits `.2` and `.5` separately.
                // Rejoin one adjacent fractional suffix; extra dots remain invalid.
                let fraction = tokens.get(at).and_then(|next| match &next.token {
                    Token::Number(value, false)
                        if right.bytes().all(|byte| byte.is_ascii_digit())
                            && value.starts_with('.') =>
                    {
                        Some(value)
                    }
                    _ => None,
                });
                let value = if let Some(fraction) = fraction {
                    at += 1;
                    format!("{right}{fraction}")
                } else {
                    right.into()
                };
                out.push(Token::Number(value, false));
            }
        } else {
            out.push(token.clone());
            at += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests;
