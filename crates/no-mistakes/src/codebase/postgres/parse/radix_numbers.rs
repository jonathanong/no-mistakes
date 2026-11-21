//! Rewrite PostgreSQL radix literals into the decimal spelling callers read.
use super::super::numeric_literal::integer;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn repair(tokens: &[TokenWithSpan]) -> Option<Vec<TokenWithSpan>> {
    let mut out: Option<Vec<TokenWithSpan>> = None;
    let mut index = 0;
    while index < tokens.len() {
        let next = tokens.get(index + 1);
        if let Some((step, token)) = rewrite_radix(&tokens[index], next) {
            let out = out.get_or_insert_with(|| tokens[..index].to_vec());
            out.push(token);
            index += step;
            continue;
        }
        if let Some(out) = &mut out {
            out.push(tokens[index].clone());
        }
        index += 1;
    }
    out
}

fn rewrite_radix(
    token: &TokenWithSpan,
    next: Option<&TokenWithSpan>,
) -> Option<(usize, TokenWithSpan)> {
    let text = radix_text(token)?;
    if let Some(broken) = invalid_suffix(token, next) {
        return Some(broken);
    }
    let value = integer(text).ok()?;
    let mut repaired = token.clone();
    repaired.token = Token::Number(value.to_string(), false);
    Some((1, repaired))
}

fn invalid_suffix(
    token: &TokenWithSpan,
    next: Option<&TokenWithSpan>,
) -> Option<(usize, TokenWithSpan)> {
    let next = next?;
    if !glued_word(token, next) {
        return None;
    }
    // A trailing identifier would parse as a projection alias.
    let mut broken = token.clone();
    broken.token = Token::Char('\0');
    broken.span.end = next.span.end;
    Some((2, broken))
}

/// sqlparser 0.64 emits `0x` / `0o` / `0b` as one number token. Downstream caps
/// still read a decimal spelling, so rewrite the value and keep the source span.
fn radix_text(token: &TokenWithSpan) -> Option<&str> {
    let Token::Number(text, _) = &token.token else {
        return None;
    };
    matches!(
        text.get(..2).map(str::as_bytes),
        Some(b"0x" | b"0X" | b"0o" | b"0O" | b"0b" | b"0B")
    )
    .then_some(text.as_str())
}

fn glued_word(token: &TokenWithSpan, next: &TokenWithSpan) -> bool {
    token.span.end == next.span.start
        && matches!(&next.token, Token::Word(word) if word.quote_style.is_none())
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
