//! Conservative recognition of complete PostgreSQL grammar absent from sqlparser.
//! Unknown shapes remain failures; a statement prefix alone is never proof.
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn valid(tokens: &[TokenWithSpan]) -> bool {
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
            cursor.identifier()
                && cursor.word("FOR")
                && (cursor.words(&["ALL", "TABLES"]) || (cursor.word("TABLE") && cursor.names()))
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
    if !cursor.word("GRANT") {
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
    cursor.word("TO")
        && cursor.identifiers()
        && (!cursor.word("WITH") || cursor.words(&["GRANT", "OPTION"]))
}

struct Cursor<'a> {
    tokens: &'a [&'a Token],
    at: usize,
}
impl Cursor<'_> {
    fn word(&mut self, value: &str) -> bool {
        if matches!(self.tokens.get(self.at), Some(Token::Word(word))
            if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(value))
        {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn words(&mut self, values: &[&str]) -> bool {
        let start = self.at;
        if values.iter().all(|value| self.word(value)) {
            true
        } else {
            self.at = start;
            false
        }
    }
    fn identifier(&mut self) -> bool {
        // Reserved keywords must not masquerade as missing names or clauses.
        if matches!(self.tokens.get(self.at), Some(Token::Word(word))
            if !word.value.is_empty() && (word.quote_style == Some('"')
                || (word.quote_style.is_none() && (word.keyword == sqlparser::keywords::Keyword::NoKeyword
                    || word.value.eq_ignore_ascii_case("public")))))
        {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn name(&mut self) -> bool {
        self.identifier() && (!self.period() || self.identifier())
    }
    fn identifiers(&mut self) -> bool {
        if !self.identifier() {
            return false;
        }
        while self.comma() {
            if !self.identifier() {
                return false;
            }
        }
        true
    }
    fn names(&mut self) -> bool {
        if !self.name() {
            return false;
        }
        while self.comma() {
            if !self.name() {
                return false;
            }
        }
        true
    }
    fn comma(&mut self) -> bool {
        if matches!(self.tokens.get(self.at), Some(Token::Comma)) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn period(&mut self) -> bool {
        if matches!(self.tokens.get(self.at), Some(Token::Period)) {
            self.at += 1;
            true
        } else {
            false
        }
    }
}
