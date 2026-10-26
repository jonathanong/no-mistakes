use super::Cursor;
use sqlparser::tokenizer::Token;
pub(super) fn parse(c: &mut Cursor<'_>) -> bool {
    if !c.word("CREATE") {
        return false;
    }
    if c.word("DATABASE") {
        return database(c);
    }
    if c.word("GROUP") {
        return c.identifier();
    }
    if c.word("STATISTICS") {
        return statistics(c);
    }
    if c.word("SUBSCRIPTION") {
        return subscription(c);
    }
    if c.word("TABLESPACE") {
        return tablespace(c);
    }
    c.words(&["OR", "REPLACE"]);
    if c.word("RULE") {
        return rule(c);
    }
    language(c)
}

fn statistics(c: &mut Cursor<'_>) -> bool {
    c.words(&["IF", "NOT", "EXISTS"]);
    if !c.name() || !c.word("ON") || !c.identifier() || !c.token(Token::Comma) || !c.identifier() {
        return false;
    }
    while c.token(Token::Comma) {
        if !c.identifier() {
            return false;
        }
    }
    c.word("FROM") && c.name()
}
fn subscription(c: &mut Cursor<'_>) -> bool {
    if !c.identifier()
        || !c.word("CONNECTION")
        || !c.string()
        || !c.word("PUBLICATION")
        || !c.identifier()
    {
        return false;
    }
    while c.token(Token::Comma) {
        if !c.identifier() {
            return false;
        }
    }
    true
}
fn tablespace(c: &mut Cursor<'_>) -> bool {
    if !c.identifier() {
        return false;
    }
    if c.word("OWNER")
        && !(c.word("CURRENT_ROLE")
            || c.word("CURRENT_USER")
            || c.word("SESSION_USER")
            || c.identifier())
    {
        return false;
    }
    c.word("LOCATION") && c.string()
}
fn rule(c: &mut Cursor<'_>) -> bool {
    c.identifier()
        && c.words(&["AS", "ON"])
        // SELECT rules require a SELECT action, so cannot use DO NOTHING.
        && (c.word("INSERT") || c.word("UPDATE") || c.word("DELETE"))
        && c.word("TO")
        && c.name()
        && c.word("DO")
        && {
            if !c.word("ALSO") {
                c.word("INSTEAD");
            }
            c.word("NOTHING")
        }
}
fn language(c: &mut Cursor<'_>) -> bool {
    c.word("TRUSTED");
    c.word("PROCEDURAL");
    if !c.word("LANGUAGE") || !c.identifier() {
        return false;
    }
    if c.word("HANDLER") {
        if !c.function_name() {
            return false;
        }
        if c.word("INLINE") && !c.function_name() {
            return false;
        }
        if c.word("VALIDATOR") && !c.function_name() {
            return false;
        }
    }
    true
}

fn database(c: &mut Cursor<'_>) -> bool {
    if !c.identifier() {
        return false;
    }
    let with = c.word("WITH");
    let mut seen = std::collections::BTreeSet::new();
    loop {
        let option = if c.word("OWNER") {
            "OWNER"
        } else if c.word("TEMPLATE") {
            "TEMPLATE"
        } else if c.word("ENCODING") {
            "ENCODING"
        } else if c.words(&["CONNECTION", "LIMIT"]) {
            "LIMIT"
        } else {
            break;
        };
        if !seen.insert(option) {
            return false;
        }
        c.token(Token::Eq);
        let valid = match option {
            "ENCODING" => c.word("DEFAULT") || c.string() || c.integer(),
            "LIMIT" => {
                c.token(Token::Minus);
                c.integer()
            }
            _ => c.word("DEFAULT") || c.identifier(),
        };
        if !valid {
            return false;
        }
    }
    !with || !seen.is_empty()
}
