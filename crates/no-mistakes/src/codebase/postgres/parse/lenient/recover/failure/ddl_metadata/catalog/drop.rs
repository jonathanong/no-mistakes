use super::super::Cursor;
use sqlparser::tokenizer::Token;

pub(super) fn parse<'a>(c: &mut Cursor<'a>) -> bool {
    if c.word("GROUP") {
        c.words(&["IF", "EXISTS"]);
        return list(c, Cursor::identifier);
    }
    // Tablespaces, unlike role groups, have exactly one unqualified name.
    if c.word("TABLESPACE") {
        c.words(&["IF", "EXISTS"]);
        return c.identifier();
    }
    if c.words(&["ACCESS", "METHOD"]) || c.word("LANGUAGE") || c.word("SUBSCRIPTION") {
        c.words(&["IF", "EXISTS"]);
        if !c.identifier() {
            return false;
        }
        c.drop_behavior();
        return true;
    }
    let item = if c.word("AGGREGATE") {
        Some(Cursor::aggregate_signature as fn(&mut Cursor<'a>) -> bool)
    } else if c.word("ROUTINE") {
        Some(Cursor::signature as fn(&mut Cursor<'a>) -> bool)
    } else if c.word("STATISTICS") {
        Some(Cursor::name as fn(&mut Cursor<'a>) -> bool)
    } else {
        None
    };
    if let Some(item) = item {
        c.words(&["IF", "EXISTS"]);
        if !list(c, item) {
            return false;
        }
        c.drop_behavior();
        return true;
    }
    if c.word("CAST") {
        c.words(&["IF", "EXISTS"]);
        if !c.token(Token::LParen)
            || !c.data_type()
            || !c.word("AS")
            || !c.data_type()
            || !c.token(Token::RParen)
        {
            return false;
        }
    } else if c.word("RULE") {
        c.words(&["IF", "EXISTS"]);
        if !c.identifier() || !c.word("ON") || !c.name() {
            return false;
        }
    } else {
        let known = c.word("CONVERSION")
            || (c.words(&["TEXT", "SEARCH"])
                && (c.word("CONFIGURATION")
                    || c.word("DICTIONARY")
                    || c.word("PARSER")
                    || c.word("TEMPLATE")));
        if !known {
            return false;
        }
        c.words(&["IF", "EXISTS"]);
        if !c.name() {
            return false;
        }
    }
    c.drop_behavior();
    true
}

fn list<'a>(c: &mut Cursor<'a>, item: fn(&mut Cursor<'a>) -> bool) -> bool {
    if !item(c) {
        return false;
    }
    while c.token(Token::Comma) {
        if !item(c) {
            return false;
        }
    }
    true
}
