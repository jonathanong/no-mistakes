use super::Cursor;
use sqlparser::tokenizer::Token;
pub(super) fn parse(c: &mut Cursor<'_>) -> bool {
    if c.words(&["CREATE", "EVENT", "TRIGGER"]) {
        if !c.identifier()
            || !c.word("ON")
            || ![
                "ddl_command_start",
                "ddl_command_end",
                "sql_drop",
                "table_rewrite",
            ]
            .iter()
            .any(|w| c.word(w))
        {
            return false;
        }
        if c.word("WHEN") {
            // PostgreSQL 17 has one supported filter variable (TAG), and
            // rejects a second TAG filter even when joined with AND.
            if !c.words(&["TAG", "IN"]) || !c.token(Token::LParen) || !c.string() {
                return false;
            }
            while c.token(Token::Comma) {
                if !c.string() {
                    return false;
                }
            }
            if !c.token(Token::RParen) {
                return false;
            }
        }
        c.word("EXECUTE")
            && (c.word("FUNCTION") || c.word("PROCEDURE"))
            && c.function_name()
            && c.token(Token::LParen)
            && c.token(Token::RParen)
    } else if c.words(&["ALTER", "EVENT", "TRIGGER"]) {
        if !c.identifier() {
            return false;
        }
        if c.word("DISABLE") {
            true
        } else if c.word("ENABLE") {
            if !c.word("REPLICA") {
                c.word("ALWAYS");
            }
            true
        } else if c.words(&["RENAME", "TO"]) {
            c.identifier()
        } else {
            c.words(&["OWNER", "TO"])
                && (c.word("CURRENT_ROLE")
                    || c.word("CURRENT_USER")
                    || c.word("SESSION_USER")
                    || c.identifier())
        }
    } else if c.words(&["DROP", "EVENT", "TRIGGER"]) {
        drop(c)
    } else {
        false
    }
}

fn drop(c: &mut Cursor<'_>) -> bool {
    c.words(&["IF", "EXISTS"]);
    if !c.identifier() {
        return false;
    }
    while c.token(Token::Comma) {
        if !c.identifier() {
            return false;
        }
    }
    c.drop_behavior();
    true
}
