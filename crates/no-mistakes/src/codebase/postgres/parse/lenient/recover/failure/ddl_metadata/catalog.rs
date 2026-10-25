use super::Cursor;
mod drop;
use sqlparser::tokenizer::Token;
pub(super) fn parse(c: &mut Cursor<'_>) -> bool {
    if c.word("DROP") {
        return drop::parse(c);
    }
    if !c.word("ALTER") {
        return false;
    }
    if c.word("SYSTEM") {
        return setting(c);
    }
    if c.word("DATABASE") {
        if !c.identifier() {
            return false;
        }
        if c.words(&["RENAME", "TO"]) {
            return c.identifier();
        }
        return setting(c);
    }
    if c.word("DOMAIN") {
        return c.name() && c.words(&["SET", "NOT", "NULL"]);
    }
    if c.word("EXTENSION") {
        return c.identifier() && c.word("UPDATE");
    }
    if c.word("GROUP") {
        return group(c);
    }
    if c.words(&["MATERIALIZED", "VIEW"]) {
        return c.name() && c.words(&["OWNER", "TO"]) && c.identifier();
    }
    if c.word("PROCEDURE") || c.word("ROUTINE") {
        return c.signature() && c.words(&["RENAME", "TO"]) && c.identifier();
    }
    if c.word("RULE") || c.word("TRIGGER") {
        return c.identifier()
            && c.word("ON")
            && c.name()
            && c.words(&["RENAME", "TO"])
            && c.identifier();
    }
    relation(c)
}

fn number(c: &mut Cursor<'_>) -> bool {
    if matches!(c.0.peek_token().token, Token::Number(ref n, false) if n.chars().all(|x| x.is_ascii_digit()))
    {
        c.0.next_token();
        true
    } else {
        false
    }
}

// Statistics permits the obsolete default target -1, but no smaller target.
// Sequence increments instead permit descending (negative) integers.
fn signed_number(c: &mut Cursor<'_>, statistics: bool) -> bool {
    let negative = c.token(Token::Minus);
    if !negative {
        c.token(Token::Plus);
    }
    if negative
        && statistics
        && !matches!(c.0.peek_token().token,
        Token::Number(ref n, false) if n.parse::<u64>().is_ok_and(|value| value <= 1))
    {
        return false;
    }
    number(c)
}

fn group(c: &mut Cursor<'_>) -> bool {
    if !c.identifier() || !(c.word("ADD") || c.word("DROP")) || !c.word("USER") || !c.identifier() {
        return false;
    }
    while c.token(Token::Comma) {
        if !c.identifier() {
            return false;
        }
    }
    true
}

fn relation(c: &mut Cursor<'_>) -> bool {
    if c.word("SEQUENCE") {
        return c.name() && c.word("INCREMENT") && {
            c.word("BY");
            signed_number(c, false)
        };
    }
    if c.word("STATISTICS") {
        return c.name()
            && c.words(&["SET", "STATISTICS"])
            && (c.word("DEFAULT") || signed_number(c, true));
    }
    if c.word("SUBSCRIPTION") {
        return c.identifier() && (c.word("ENABLE") || c.word("DISABLE"));
    }
    if c.word("LANGUAGE") || c.word("TABLESPACE") {
        return c.identifier() && c.words(&["RENAME", "TO"]) && c.identifier();
    }
    if c.word("CONVERSION") || c.word("VIEW") {
        return c.name() && c.words(&["RENAME", "TO"]) && c.identifier();
    }
    false
}

fn setting(c: &mut Cursor<'_>) -> bool {
    if c.word("RESET") {
        return c.word("ALL") || c.name();
    }
    if !c.word("SET") || !c.name() || !(c.word("TO") || c.token(Token::Eq)) {
        return false;
    }
    loop {
        if !(c.string()
            || c.word("DEFAULT")
            || c.word("TRUE")
            || c.word("FALSE")
            || c.word("ON")
            || c.word("OFF")
            || number(c)
            || c.identifier())
        {
            return false;
        }
        if !c.token(Token::Comma) {
            return true;
        }
    }
}
