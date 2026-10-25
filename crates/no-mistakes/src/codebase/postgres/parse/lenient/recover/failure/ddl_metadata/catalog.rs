use super::Cursor;
use sqlparser::tokenizer::Token;
pub(super) fn parse(c: &mut Cursor<'_>) -> bool {
    if c.word("DROP") {
        return drop(c);
    }
    if !c.word("ALTER") {
        return false;
    }
    if c.word("DOMAIN") {
        return c.name() && c.words(&["SET", "NOT", "NULL"]);
    }
    if c.word("EXTENSION") {
        return c.identifier() && c.word("UPDATE");
    }
    if c.word("GROUP") {
        if !c.identifier()
            || !(c.word("ADD") || c.word("DROP"))
            || !c.word("USER")
            || !c.identifier()
        {
            return false;
        }
        while c.token(Token::Comma) {
            if !c.identifier() {
                return false;
            }
        }
        return true;
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
    if c.word("SEQUENCE") {
        return c.name() && c.word("INCREMENT") && {
            c.word("BY");
            number(c)
        };
    }
    if c.word("STATISTICS") {
        return c.name() && c.words(&["SET", "STATISTICS"]) && number(c);
    }
    if c.word("SUBSCRIPTION") {
        return c.identifier() && (c.word("ENABLE") || c.word("DISABLE"));
    }
    if c.word("DATABASE") || c.word("LANGUAGE") || c.word("TABLESPACE") {
        return c.identifier() && c.words(&["RENAME", "TO"]) && c.identifier();
    }
    if c.word("CONVERSION") || c.word("VIEW") {
        return c.name() && c.words(&["RENAME", "TO"]) && c.identifier();
    }
    false
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
fn drop(c: &mut Cursor<'_>) -> bool {
    // These objects live outside schemas. Roles/tablespaces also have no
    // CASCADE/RESTRICT suffix, unlike the schema-scoped DROP forms below.
    if c.word("GROUP") || c.word("TABLESPACE") {
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
    let signature = if c.word("AGGREGATE") || c.word("ROUTINE") {
        true
    } else if c.word("CAST") {
        c.words(&["IF", "EXISTS"]);
        if !c.token(Token::LParen)
            || !c.data_type()
            || !c.word("AS")
            || !c.data_type()
            || !c.token(Token::RParen)
        {
            return false;
        }
        c.drop_behavior();
        return true;
    } else if c.word("RULE") {
        c.words(&["IF", "EXISTS"]);
        if !c.identifier() || !c.word("ON") || !c.name() {
            return false;
        }
        c.drop_behavior();
        return true;
    } else {
        let known = c.word("CONVERSION")
            || c.word("STATISTICS")
            || (c.words(&["TEXT", "SEARCH"])
                && (c.word("CONFIGURATION")
                    || c.word("DICTIONARY")
                    || c.word("PARSER")
                    || c.word("TEMPLATE")));
        if !known {
            return false;
        }
        false
    };
    c.words(&["IF", "EXISTS"]);
    if !(if signature { c.signature() } else { c.name() }) {
        return false;
    }
    c.drop_behavior();
    true
}
