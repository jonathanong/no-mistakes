//! Additional complete ALTER object grammars used by PostgreSQL regression SQL.
use super::Cursor;
use sqlparser::tokenizer::Token;

pub(super) fn parse(c: &mut Cursor<'_>) -> bool {
    if !c.word("ALTER") {
        return false;
    }
    if c.word("AGGREGATE") {
        return aggregate(c);
    }
    if !c.word("OPERATOR") {
        return false;
    }
    operator(c)
}
fn aggregate(c: &mut Cursor<'_>) -> bool {
    if !c.function_name() || !c.token(Token::LParen) || !args(c) {
        return false;
    }
    if c.words(&["ORDER", "BY"]) && !args(c) {
        return false;
    }
    c.token(Token::RParen) && c.words(&["RENAME", "TO"]) && c.identifier()
}
fn args(c: &mut Cursor<'_>) -> bool {
    if !arg(c) {
        return false;
    }
    while c.token(Token::Comma) {
        if !arg(c) {
            return false;
        }
    }
    true
}
fn arg(c: &mut Cursor<'_>) -> bool {
    let _ = c.word("VARIADIC") || c.word("IN") || c.word("OUT");
    c.data_type()
}
fn operator(c: &mut Cursor<'_>) -> bool {
    symbol(c)
        && c.token(Token::LParen)
        && c.data_type()
        && c.token(Token::Comma)
        && c.data_type()
        && c.token(Token::RParen)
        && ((c.words(&["OWNER", "TO"]) && c.identifier())
            || (c.words(&["SET", "SCHEMA"]) && c.identifier()))
}
fn symbol(c: &mut Cursor<'_>) -> bool {
    let mut spelling = String::new();
    let mut previous_end = None;
    loop {
        let token = c.0.peek_token();
        let string = token.token.to_string();
        if string.is_empty()
            || !string.chars().all(|ch| "+-*/<>=~!@#%^&|`?".contains(ch))
            || previous_end.is_some_and(|end| end != token.span.start)
        {
            break;
        }
        previous_end = Some(token.span.end);
        spelling.push_str(&string);
        c.0.next_token();
    }
    !spelling.is_empty()
        && !(spelling.len() > 1
            && spelling.ends_with(['+', '-'])
            && !spelling.chars().any(|ch| "~!@#%^&|`?".contains(ch)))
}
