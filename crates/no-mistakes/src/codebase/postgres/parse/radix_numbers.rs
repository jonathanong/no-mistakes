//! Recover radix numbers split into adjacent number/word tokens by sqlparser.
use super::super::numeric_literal::integer;
use sqlparser::tokenizer::{Token, TokenWithSpan};
use std::num::IntErrorKind;

pub(super) fn repair(tokens: &[TokenWithSpan]) -> Option<Vec<TokenWithSpan>> {
    let mut out: Option<Vec<TokenWithSpan>> = None;
    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
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
