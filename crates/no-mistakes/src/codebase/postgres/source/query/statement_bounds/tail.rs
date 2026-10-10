//! A statement ends at the last token that does not close a parenthesis opened
//! outside it. Those closers are CTE-body wrappers, not statement syntax.

use super::Locations;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn open_depth(prefix: &str) -> Result<i32, ()> {
    let mut depth = 0i32;
    for token in covered_tokens(prefix)? {
        match token.token {
            Token::LParen => depth += 1,
            Token::RParen => {
                depth -= 1;
                if depth < 0 {
                    return Err(());
                }
            }
            _ => {}
        }
    }
    Ok(depth)
}

/// Byte offset of the last statement token in `tail`, relative to `tail`.
/// `Ok(None)` means `tail` adds no statement token. `Err` when `tail` does not
/// tokenize through its last byte.
pub(super) fn bounded_end(tail: &str, mut depth: i32) -> Result<Option<usize>, ()> {
    if tail.is_empty() {
        return Ok(None);
    }
    let tokens = covered_tokens(tail)?;
    let local = Locations::new(tail);
    let mut end = None;
    for token in &tokens {
        if matches!(token.token, Token::RParen) && depth == 0 {
            break;
        }
        match token.token {
            Token::LParen => depth += 1,
            Token::RParen => depth -= 1,
            _ => {}
        }
        if matches!(token.token, Token::Whitespace(_) | Token::EOF) {
            continue;
        }
        end = Some(local.position(token.span.end).ok_or(())?.offset);
    }
    Ok(end)
}

fn covered_tokens(text: &str) -> Result<Vec<TokenWithSpan>, ()> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let tokens = crate::codebase::postgres::parse::operator_boundary::tokenize_with_location(
        &PostgreSqlDialect {},
        text,
    )
    .map_err(|_| ())?;
    let local = Locations::new(text);
    let consumed = tokens
        .last()
        .and_then(|token| local.position(token.span.end))
        .map(|position| position.offset);
    accept_consumed(consumed, text.len())?;
    Ok(tokens)
}

pub(super) fn accept_consumed(consumed: Option<usize>, len: usize) -> Result<(), ()> {
    if consumed == Some(len) {
        Ok(())
    } else {
        Err(())
    }
}
