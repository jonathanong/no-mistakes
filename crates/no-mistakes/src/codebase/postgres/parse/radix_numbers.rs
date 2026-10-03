//! Recover radix numbers split into adjacent number/word tokens by sqlparser.
use super::super::numeric_literal::integer;
use sqlparser::tokenizer::{Token, TokenWithSpan};
use std::num::IntErrorKind;

pub(super) fn repair(tokens: &[TokenWithSpan]) -> Option<Vec<TokenWithSpan>> {
    let mut out = Vec::with_capacity(tokens.len());
    let mut index = 0;
    let mut changed = false;
    while index < tokens.len() {
        let token = &tokens[index];
        if matches!(&token.token, Token::Number(number, _) if number == "0") {
            if let Some(next) = tokens.get(index + 1) {
                if let Token::Word(word) = &next.token {
                    if word.quote_style.is_none()
                        && token.span.end == next.span.start
                        && matches!(
                            word.value.as_bytes().first(),
                            Some(b'o' | b'O' | b'b' | b'B')
                        )
                    {
                        let raw = format!("0{}", word.value);
                        let value = match integer(&raw) {
                            Ok(value) => Some(value.to_string()),
                            Err(error)
                                if *error.kind() == IntErrorKind::PosOverflow
                                    && raw[2..].chars().filter(|c| *c != '_').all(|c| {
                                        c.is_digit(
                                            if raw.as_bytes()[1].eq_ignore_ascii_case(&b'b') {
                                                2
                                            } else {
                                                8
                                            },
                                        )
                                    }) =>
                            {
                                Some(raw)
                            }
                            Err(_) => None,
                        };
                        if let Some(value) = value {
                            let mut combined = token.clone();
                            combined.token = Token::Number(value, false);
                            combined.span.end = next.span.end;
                            out.push(combined);
                            index += 2;
                            changed = true;
                            continue;
                        }
                    }
                }
            }
        }
        out.push(token.clone());
        index += 1;
    }
    changed.then_some(out)
}

#[cfg(test)]
mod tests;
