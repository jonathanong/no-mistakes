//! Conservative recognition of complete PostgreSQL grammar absent from sqlparser.
//! Unknown shapes remain failures; a statement prefix alone is never proof.
use sqlparser::tokenizer::{Token, TokenWithSpan};
mod cursor;
mod native;
mod publication;
use cursor::Cursor;

pub(super) fn valid(tokens: &[TokenWithSpan]) -> bool {
    if native::valid(tokens) {
        return true;
    }
    let tokens = tokens
        .iter()
        .map(|item| &item.token)
        .filter(|token| !matches!(token, Token::Whitespace(_)))
        .collect::<Vec<_>>();
    let mut cursor = Cursor {
        tokens: &tokens,
        at: 0,
    };
    let recognized = if cursor.words(&["REFRESH", "MATERIALIZED", "VIEW"]) {
        cursor.word("CONCURRENTLY");
        cursor.name()
            && (!cursor.word("WITH") || {
                cursor.word("NO");
                cursor.word("DATA")
            })
    } else {
        cursor.at = 0;
        if cursor.words(&["CREATE", "PUBLICATION"]) {
            publication::valid(&mut cursor)
        } else if cursor.words(&["ALTER", "PUBLICATION"]) {
            publication::alter(&mut cursor)
        } else if cursor.words(&["DROP", "PUBLICATION"]) {
            publication::drop(&mut cursor)
        } else {
            cursor.at = 0;
            default_privileges(&mut cursor)
        }
    };
    recognized && cursor.at == tokens.len()
}

fn default_privileges(cursor: &mut Cursor<'_>) -> bool {
    if !cursor.words(&["ALTER", "DEFAULT", "PRIVILEGES"]) {
        return false;
    }
    if cursor.word("FOR")
        && (!(cursor.word("ROLE") || cursor.word("USER")) || !cursor.identifiers())
    {
        return false;
    }
    if cursor.words(&["IN", "SCHEMA"]) && !cursor.identifiers() {
        return false;
    }
    let revoke = cursor.word("REVOKE");
    if revoke {
        if cursor.word("GRANT") && !cursor.words(&["OPTION", "FOR"]) {
            return false;
        }
    } else if !cursor.word("GRANT") {
        return false;
    }
    if cursor.word("ALL") {
        cursor.word("PRIVILEGES");
    } else {
        loop {
            if ![
                "SELECT",
                "INSERT",
                "UPDATE",
                "DELETE",
                "TRUNCATE",
                "REFERENCES",
                "TRIGGER",
                "USAGE",
                "CREATE",
                "EXECUTE",
                "MAINTAIN",
            ]
            .iter()
            .any(|word| cursor.word(word))
            {
                return false;
            }
            if !cursor.comma() {
                break;
            }
        }
    }
    if !cursor.word("ON")
        || ![
            "TABLES",
            "SEQUENCES",
            "FUNCTIONS",
            "ROUTINES",
            "TYPES",
            "SCHEMAS",
        ]
        .iter()
        .any(|word| cursor.word(word))
    {
        return false;
    }
    cursor.word(if revoke { "FROM" } else { "TO" })
        && cursor.identifiers()
        && if revoke {
            if !cursor.word("CASCADE") {
                cursor.word("RESTRICT");
            }
            true
        } else {
            !cursor.word("WITH") || cursor.words(&["GRANT", "OPTION"])
        }
}
