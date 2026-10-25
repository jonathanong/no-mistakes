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
    if !c.type_name() {
        return false;
    }
    while c.comma() {
        if !c.type_name() {
            return false;
        }
    }
    c.right_paren()
}
