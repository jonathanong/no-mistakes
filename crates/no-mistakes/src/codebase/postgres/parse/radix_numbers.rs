//! Recover radix numbers split into adjacent number/word tokens by sqlparser.
use super::super::numeric_literal::integer;
use sqlparser::tokenizer::{Token, TokenWithSpan};
use std::num::IntErrorKind;

pub(super) fn repair(tokens: &[TokenWithSpan]) -> Option<Vec<TokenWithSpan>> {
    let mut out: Option<Vec<TokenWithSpan>> = None;
    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
        if radix_prefix(token) {
            if let Some(next) = tokens.get(index + 1) {
                if glued_word(token, next) {
                    let out = out.get_or_insert_with(|| tokens[..index].to_vec());
                    let mut broken = token.clone();
                    broken.token = Token::Char('\0');
                    broken.span.end = next.span.end;
                    out.push(broken);
                    index += 2;
                    continue;
                }
            }
            if let Some(value) = radix_integer(token) {
                let out = out.get_or_insert_with(|| tokens[..index].to_vec());
                let mut repaired = token.clone();
                repaired.token = Token::Number(value, false);
                out.push(repaired);
                index += 1;
                continue;
            }
        }
        if let Some(value) = numeric_hex(token) {
            let out = out.get_or_insert_with(|| tokens[..index].to_vec());
            let mut repaired = token.clone();
            repaired.token = Token::Number(value, false);
            out.push(repaired);
            index += 1;
            continue;
        }
        if matches!(&token.token, Token::Number(number, _) if number == "0") {
            if let Some(next) = tokens.get(index + 1) {
                if let Token::Word(word) = &next.token {
                    if word.quote_style.is_none()
                        && token.span.end == next.span.start
                        && matches!(
                            word.value.as_bytes().first(),
                            Some(b'o' | b'O' | b'b' | b'B' | b'x' | b'X')
                        )
                    {
                        let raw = format!("0{}", word.value);
                        let hex = matches!(raw.as_bytes()[1], b'x' | b'X');
                        let value = match integer(&raw) {
                            Ok(value) => Some(value.to_string()),
                            Err(error)
                                if *error.kind() == IntErrorKind::PosOverflow
                                    && raw[2..].chars().filter(|c| *c != '_').all(|c| {
                                        c.is_digit(match raw.as_bytes()[1].to_ascii_lowercase() {
                                            b'b' => 2,
                                            b'o' => 8,
                                            _ => 16,
                                        })
                                    }) =>
                            {
                                Some(raw)
                            }
                            Err(_) => None,
                        };
                        let repaired =
                            value.map(|value| Token::Number(value, false)).or_else(|| {
                                // Keep malformed contiguous hex lexically invalid in every position.
                                // Leaving the pair untouched would parse as a projection alias.
                                hex.then_some(Token::Char('\0'))
                            });
                        if let Some(repaired) = repaired {
                            let mut combined = token.clone();
                            combined.token = repaired;
                            combined.span.end = next.span.end;
                            // Ordinary SQL keeps its original token vector without cloning.
                            let out = out.get_or_insert_with(|| {
                                let mut repaired = Vec::with_capacity(tokens.len());
                                repaired.extend_from_slice(&tokens[..index]);
                                repaired
                            });
                            out.push(combined);
                            index += 2;
                            continue;
                        }
                    }
                }
            }
        }
        if let Some(out) = &mut out {
            out.push(token.clone());
        }
        index += 1;
    }
    out
}

/// sqlparser 0.64 emits `0x` / `0o` / `0b` as one number token. Downstream caps
/// still read a decimal spelling, so rewrite the value and keep the source span.
fn radix_prefix(token: &TokenWithSpan) -> bool {
    let Token::Number(text, _) = &token.token else {
        return false;
    };
    matches!(
        text.get(..2).map(str::as_bytes),
        Some(b"0x" | b"0X" | b"0o" | b"0O" | b"0b" | b"0B")
    )
}

fn glued_word(token: &TokenWithSpan, next: &TokenWithSpan) -> bool {
    token.span.end == next.span.start
        && matches!(&next.token, Token::Word(word) if word.quote_style.is_none())
}

fn radix_integer(token: &TokenWithSpan) -> Option<String> {
    let Token::Number(text, _) = &token.token else {
        return None;
    };
    match integer(text) {
        Ok(value) => Some(value.to_string()),
        Err(_) => None,
    }
}

// sqlparser uses the same token variant for 0xFF and X'FF'. Their lexer spans retain
// the distinction: numeric spelling has two prefix characters; quoted spelling has
// a prefix and two quote characters. Normalize while that lexical identity survives,
// before migration recovery reconstructs source text and changes its byte provenance.
fn numeric_hex(token: &TokenWithSpan) -> Option<String> {
    let Token::HexStringLiteral(value) = &token.token else {
        return None;
    };
    let span = token.span;
    if span.start.line != span.end.line
        || span.end.column
            != span
                .start
                .column
                .saturating_add(value.chars().count() as u64 + 2)
    {
        return None;
    }
    let raw = format!("0x{value}");
    match integer(&raw) {
        Ok(value) => Some(value.to_string()),
        Err(error) if *error.kind() == IntErrorKind::PosOverflow => Some(raw),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests;
