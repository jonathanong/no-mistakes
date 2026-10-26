//! Validate loop headers separately from the SQL action sharing their chunk.
use super::{control_end, next, word};
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn action_start(tokens: &[Token], mut at: usize) -> Result<Option<usize>, ()> {
    let mut looped = false;
    loop {
        let start = skip_label(tokens, at);
        let Some(first) = tokens.get(start) else {
            return Ok(looped.then_some(at));
        };
        if looped && (word(first, "BEGIN") || word(first, "ELSE")) {
            at = next(tokens, start + 1);
            continue;
        }
        if looped && (word(first, "IF") || word(first, "ELSIF")) {
            let then = control_end(tokens, start + 1, "THEN").ok_or(())?;
            if !valid_control(&tokens[at..=then]) {
                return Err(());
            }
            at = next(tokens, then + 1);
            continue;
        }
        let end = if word(first, "LOOP") {
            start
        } else if ["FOR", "WHILE", "FOREACH"]
            .iter()
            .any(|name| word(first, name))
        {
            control_end(tokens, start + 1, "LOOP").ok_or(())?
        } else {
            return Ok(looped.then_some(at));
        };
        if !valid_control(&tokens[at..=end]) {
            return Err(());
        }
        looped = true;
        at = next(tokens, end + 1);
    }
}

fn valid_control(tokens: &[Token]) -> bool {
    let located = tokens
        .iter()
        .cloned()
        .map(TokenWithSpan::wrap)
        .collect::<Vec<_>>();
    super::super::procedural::BodyState::default().control_chunk(&located)
}

fn skip_label(tokens: &[Token], at: usize) -> usize {
    if tokens.get(at) != Some(&Token::ShiftLeft) {
        return at;
    }
    let label = next(tokens, at + 1);
    let close = next(tokens, label + 1);
    if matches!(tokens.get(label), Some(Token::Word(_)))
        && tokens.get(close) == Some(&Token::ShiftRight)
    {
        next(tokens, close + 1)
    } else {
        at
    }
}

pub(in super::super) fn loop_action_start(tokens: &[Token]) -> Result<Option<usize>, ()> {
    let mut at = next(tokens, 0);
    let mut prefixed = false;
    loop {
        let start = skip_label(tokens, at);
        let Some(first) = tokens.get(start) else {
            return Ok(None);
        };
        if word(first, "BEGIN") || word(first, "ELSE") {
            at = next(tokens, start + 1);
        } else if word(first, "IF") || word(first, "ELSIF") {
            let Some(then) = control_end(tokens, start + 1, "THEN") else {
                return Ok(None);
            };
            at = next(tokens, then + 1);
        } else {
            break;
        }
        prefixed = true;
    }
    let action = action_start(tokens, at)?;
    // Ordinary IF recovery retains its existing semantics; validate these
    // prefixes only when this is the shared loop-specific recovery path.
    if action.is_some() && prefixed && !valid_control(&tokens[..at]) {
        return Err(());
    }
    Ok(action)
}
