use super::super::cursor::Cursor;
use sqlparser::tokenizer::Token;

pub(super) fn qualified_column(c: &mut Cursor<'_>) -> bool {
    if !c.identifier() {
        return false;
    }
    let mut separators = 0;
    while c.period() {
        if !c.identifier() {
            return false;
        }
        separators += 1;
    }
    (1..=2).contains(&separators)
}

pub(super) fn signature(c: &mut Cursor<'_>, required: bool) -> bool {
    if !c.left_paren() {
        return !required;
    }
    if c.right_paren() {
        return !required;
    }
    if c.punctuation(Token::Mul) {
        return required && c.right_paren();
    }
    if !argument(c) {
        return false;
    }
    while c.comma() {
        if !argument(c) {
            return false;
        }
    }
    c.right_paren()
}

fn argument(c: &mut Cursor<'_>) -> bool {
    // Modes are independent of the optional name; only a complete type wins.
    if !(c.word("IN") || c.word("OUT") || c.word("INOUT")) {
        c.word("VARIADIC");
    }
    let start = c.at;
    if c.type_name() && argument_end(c) {
        return true;
    }
    c.at = start;
    c.identifier() && c.type_name() && argument_end(c)
}

fn argument_end(c: &Cursor<'_>) -> bool {
    matches!(c.tokens.get(c.at), Some(Token::Comma | Token::RParen))
}
