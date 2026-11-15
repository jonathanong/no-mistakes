use sqlparser::tokenizer::{Token, Word};
#[path = "expression_index_cast_types.rs"]
mod types;

pub(super) fn is_operand(tokens: &[Token]) -> bool {
    let mut index = 0;
    if !consume_primary(tokens, &mut index) {
        return false;
    }
    while index < tokens.len() && tokens[index] == Token::DoubleColon {
        index += 1;
        if !types::consume_type_name(tokens, &mut index) {
            return false;
        }
    }
    index == tokens.len()
}

fn consume_primary(tokens: &[Token], index: &mut usize) -> bool {
    let start = *index;
    let ok = match tokens.get(*index) {
        Some(Token::LParen) => consume_pair(tokens, index, &Token::LParen, &Token::RParen),
        Some(
            Token::Number(_, _)
            | Token::Placeholder(_)
            | Token::SingleQuotedString(_)
            | Token::EscapedStringLiteral(_)
            | Token::DollarQuotedString(_)
            | Token::UnicodeStringLiteral(_)
            | Token::NationalStringLiteral(_)
            | Token::HexStringLiteral(_),
        ) => {
            *index += 1;
            true
        }
        Some(Token::Word(word)) if bare_word(word, "case") => consume_case(tokens, index),
        Some(Token::Word(word)) if bare_word(word, "array") => consume_array(tokens, index),
        Some(Token::Word(word)) if bare_word(word, "exists") => {
            *index += 1;
            consume_pair(tokens, index, &Token::LParen, &Token::RParen)
        }
        Some(Token::Word(_)) => consume_identifier(tokens, index),
        _ => false,
    };
    if !ok || !consume_brackets(tokens, index) {
        *index = start;
        return false;
    }
    true
}

fn consume_identifier(tokens: &[Token], index: &mut usize) -> bool {
    *index += 1;
    while tokens.get(*index) == Some(&Token::Period) {
        *index += 1;
        if !consume_word(tokens, index) {
            return false;
        }
    }
    tokens.get(*index) != Some(&Token::LParen)
        || consume_pair(tokens, index, &Token::LParen, &Token::RParen)
}

fn consume_array(tokens: &[Token], index: &mut usize) -> bool {
    *index += 1;
    if tokens.get(*index) == Some(&Token::LBracket) {
        consume_pair(tokens, index, &Token::LBracket, &Token::RBracket)
    } else if tokens.get(*index) == Some(&Token::LParen) {
        consume_pair(tokens, index, &Token::LParen, &Token::RParen)
    } else {
        false
    }
}

fn consume_case(tokens: &[Token], index: &mut usize) -> bool {
    let mut depth = 0;
    while *index < tokens.len() {
        if bare(tokens, *index, "case") {
            depth += 1;
        } else if bare(tokens, *index, "end") {
            depth -= 1;
            *index += 1;
            if depth == 0 {
                return true;
            }
            continue;
        }
        *index += 1;
    }
    false
}

pub(super) fn consume_brackets(tokens: &[Token], index: &mut usize) -> bool {
    while tokens.get(*index) == Some(&Token::LBracket) {
        if !consume_pair(tokens, index, &Token::LBracket, &Token::RBracket) {
            return false;
        }
    }
    true
}

pub(super) fn consume_pair(
    tokens: &[Token],
    index: &mut usize,
    open: &Token,
    close: &Token,
) -> bool {
    if tokens.get(*index) != Some(open) {
        return false;
    }
    let mut depth = 0;
    let start = *index;
    while *index < tokens.len() {
        if &tokens[*index] == open {
            depth += 1;
        } else if &tokens[*index] == close {
            depth -= 1;
            if depth == 0 {
                *index += 1;
                return true;
            }
        }
        *index += 1;
    }
    *index = start;
    false
}

pub(super) fn consume_word(tokens: &[Token], index: &mut usize) -> bool {
    if matches!(tokens.get(*index), Some(Token::Word(_))) {
        *index += 1;
        true
    } else {
        false
    }
}

pub(super) fn bare(tokens: &[Token], index: usize, value: &str) -> bool {
    matches!(tokens.get(index), Some(Token::Word(word)) if bare_word(word, value))
}

fn bare_word(word: &Word, value: &str) -> bool {
    word.quote_style.is_none() && word.value.eq_ignore_ascii_case(value)
}
