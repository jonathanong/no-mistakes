use super::{boolean, columns, number, string, token, Cursor, Token};

pub(super) fn valid(cursor: &mut Cursor<'_>) -> bool {
    if cursor.words(&["COMMENT", "ON"]) {
        return comment(cursor);
    }
    let revoke = cursor.word("REVOKE");
    if !revoke && !cursor.word("GRANT") {
        return false;
    }
    let objects = cursor.tokens[cursor.at..].iter().any(|token|
        matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case("ON")));
    if revoke {
        if objects {
            if cursor.word("GRANT") && !cursor.words(&["OPTION", "FOR"]) {
                return false;
            }
        } else if ["ADMIN", "INHERIT", "SET"]
            .iter()
            .any(|word| cursor.word(word))
            && !cursor.words(&["OPTION", "FOR"])
        {
            return false;
        }
    }
    if objects {
        if !privileges(cursor) || !cursor.word("ON") || !targets(cursor) {
            return false;
        }
    } else if !cursor.identifiers() {
        return false;
    }
    if !cursor.word(if revoke { "FROM" } else { "TO" }) || !roles(cursor) {
        return false;
    }
    if revoke {
        if !cursor.word("CASCADE") {
            cursor.word("RESTRICT");
        }
    } else if cursor.word("WITH") {
        if objects {
            if !cursor.words(&["GRANT", "OPTION"]) {
                return false;
            }
        } else {
            loop {
                if !["ADMIN", "INHERIT", "SET"]
                    .iter()
                    .any(|word| cursor.word(word))
                    || !(cursor.word("OPTION") || boolean(cursor))
                {
                    return false;
                }
                if !cursor.comma() {
                    break;
                }
            }
        }
    }
    !cursor.words(&["GRANTED", "BY"]) || role(cursor)
}

fn privileges(cursor: &mut Cursor<'_>) -> bool {
    loop {
        let all = cursor.word("ALL");
        let column = all
            || ["SELECT", "INSERT", "UPDATE", "REFERENCES"]
                .iter()
                .any(|name| cursor.word(name));
        if all {
            cursor.word("PRIVILEGES");
        }
        if !column
            && ![
                "DELETE",
                "TRUNCATE",
                "TRIGGER",
                "MAINTAIN",
                "USAGE",
                "CREATE",
                "CONNECT",
                "TEMP",
                "TEMPORARY",
                "EXECUTE",
                "SET",
            ]
            .iter()
            .any(|name| cursor.word(name))
        {
            return false;
        }
        if matches!(cursor.tokens.get(cursor.at), Some(Token::LParen))
            && (!column || !columns(cursor))
        {
            return false;
        }
        if !cursor.comma() {
            return true;
        }
    }
}

fn targets(cursor: &mut Cursor<'_>) -> bool {
    if cursor.word("ALL") {
        return ["TABLES", "SEQUENCES", "FUNCTIONS", "PROCEDURES", "ROUTINES"]
            .iter()
            .any(|name| cursor.word(name))
            && cursor.words(&["IN", "SCHEMA"])
            && cursor.names();
    }
    if cursor.words(&["LARGE", "OBJECT"]) {
        if !number(cursor, false) {
            return false;
        }
        while cursor.comma() {
            if !number(cursor, false) {
                return false;
            }
        }
        return true;
    }
    if cursor.word("FOREIGN") && !(cursor.words(&["DATA", "WRAPPER"]) || cursor.word("SERVER")) {
        return false;
    }
    [
        "TABLE",
        "SEQUENCE",
        "DATABASE",
        "DOMAIN",
        "LANGUAGE",
        "SCHEMA",
        "TABLESPACE",
        "TYPE",
        "PARAMETER",
    ]
    .iter()
    .any(|name| cursor.word(name));
    cursor.names()
}

fn role(cursor: &mut Cursor<'_>) -> bool {
    cursor.word("PUBLIC")
        || cursor.word("CURRENT_ROLE")
        || cursor.word("CURRENT_USER")
        || cursor.word("SESSION_USER")
        || cursor.identifier()
}
fn roles(cursor: &mut Cursor<'_>) -> bool {
    loop {
        cursor.word("GROUP");
        if !role(cursor) {
            return false;
        }
        if !cursor.comma() {
            return true;
        }
    }
}

fn comment(cursor: &mut Cursor<'_>) -> bool {
    if cursor.word("AGGREGATE") {
        if !cursor.name()
            || !token(cursor, Token::LParen)
            || !token(cursor, Token::Mul)
            || !token(cursor, Token::RParen)
        {
            return false;
        }
    } else if !cursor.word("CONSTRAINT")
        || !cursor.identifier()
        || !cursor.word("ON")
        || !cursor.name()
    {
        return false;
    }
    cursor.word("IS") && (cursor.word("NULL") || string(cursor))
}
