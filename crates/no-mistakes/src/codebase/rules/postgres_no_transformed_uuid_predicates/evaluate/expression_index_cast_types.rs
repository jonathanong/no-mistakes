use super::{bare, consume_brackets, consume_pair, consume_word};
use sqlparser::tokenizer::Token;

pub(super) fn consume_type_name(tokens: &[Token], index: &mut usize) -> bool {
    let start = *index;
    if consume_type_head(tokens, index) && consume_brackets(tokens, index) {
        return true;
    }
    *index = start;
    false
}

fn consume_type_head(tokens: &[Token], index: &mut usize) -> bool {
    let Some(first) = word_text(tokens, *index) else {
        return false;
    };
    let first = first.to_string();
    *index += 1;
    if tokens.get(*index) == Some(&Token::Period) {
        while tokens.get(*index) == Some(&Token::Period) {
            *index += 1;
            if !consume_word(tokens, index) {
                return false;
            }
        }
        return maybe_precision(tokens, index);
    }
    match first.as_str() {
        "double" => {
            if bare(tokens, *index, "precision") {
                *index += 1;
            }
            true
        }
        "character" | "char" | "bit" | "national" => {
            if bare(tokens, *index, "character") {
                *index += 1;
            }
            if bare(tokens, *index, "varying") {
                *index += 1;
            }
            maybe_precision(tokens, index)
        }
        "timestamp" | "time" => consume_time_type(tokens, index),
        _ => maybe_precision(tokens, index),
    }
}

fn consume_time_type(tokens: &[Token], index: &mut usize) -> bool {
    let _ = maybe_precision(tokens, index);
    if bare(tokens, *index, "with") || bare(tokens, *index, "without") {
        let mark = *index;
        *index += 1;
        if bare(tokens, *index, "time") && bare(tokens, *index + 1, "zone") {
            *index += 2;
        } else {
            *index = mark;
        }
    }
    true
}

fn maybe_precision(tokens: &[Token], index: &mut usize) -> bool {
    if tokens.get(*index) == Some(&Token::LParen) {
        consume_pair(tokens, index, &Token::LParen, &Token::RParen)
    } else {
        true
    }
}

fn word_text(tokens: &[Token], index: usize) -> Option<&str> {
    match tokens.get(index) {
        Some(Token::Word(word)) => Some(word.value.as_str()),
        _ => None,
    }
}
