use super::{boolean, columns, number, token, Cursor, Token};

pub(super) fn valid(cursor: &mut Cursor<'_>) -> bool {
    if cursor.word("REINDEX") {
        return reindex(cursor);
    }
    if cursor.word("VACUUM") {
        return vacuum(cursor);
    }
    if !cursor.word("CLUSTER") {
        return false;
    }
    let options = token(cursor, Token::LParen);
    if options {
        if !cursor.word("VERBOSE") {
            return false;
        }
        boolean(cursor);
        if !token(cursor, Token::RParen) {
            return false;
        }
    } else {
        cursor.word("VERBOSE");
    }
    if cursor.at == cursor.tokens.len() {
        return true;
    }
    cursor.name()
        && if cursor.word("ON") {
            !options && cursor.name()
        } else {
            !cursor.word("USING") || cursor.name()
        }
}

fn reindex(cursor: &mut Cursor<'_>) -> bool {
    if token(cursor, Token::LParen) {
        loop {
            if cursor.word("TABLESPACE") {
                if !cursor.identifier() {
                    return false;
                }
            } else if cursor.word("CONCURRENTLY") || cursor.word("VERBOSE") {
                boolean(cursor);
            } else {
                return false;
            }
            if !cursor.comma() {
                break;
            }
        }
        if !token(cursor, Token::RParen) {
            return false;
        }
    }
    let optional = cursor.word("SYSTEM") || cursor.word("DATABASE");
    if !optional && !(cursor.word("INDEX") || cursor.word("TABLE") || cursor.word("SCHEMA")) {
        return false;
    }
    cursor.word("CONCURRENTLY");
    optional && cursor.at == cursor.tokens.len() || cursor.name()
}

fn vacuum(cursor: &mut Cursor<'_>) -> bool {
    if token(cursor, Token::LParen) {
        loop {
            if cursor.word("PARALLEL") || cursor.word("BUFFER_USAGE_LIMIT") {
                if !number(cursor, true) {
                    return false;
                }
            } else if cursor.word("INDEX_CLEANUP") {
                if !cursor.word("AUTO") {
                    boolean(cursor);
                }
            } else if [
                "FULL",
                "FREEZE",
                "VERBOSE",
                "ANALYZE",
                "DISABLE_PAGE_SKIPPING",
                "SKIP_LOCKED",
                "PROCESS_MAIN",
                "PROCESS_TOAST",
                "TRUNCATE",
                "SKIP_DATABASE_STATS",
                "ONLY_DATABASE_STATS",
            ]
            .iter()
            .any(|name| cursor.word(name))
            {
                boolean(cursor);
            } else {
                return false;
            }
            if !cursor.comma() {
                break;
            }
        }
        if !token(cursor, Token::RParen) {
            return false;
        }
    } else {
        while ["FULL", "FREEZE", "VERBOSE", "ANALYZE"]
            .iter()
            .any(|name| cursor.word(name))
        {}
    }
    if cursor.at == cursor.tokens.len() {
        return true;
    }
    loop {
        if !cursor.name() {
            return false;
        }
        if matches!(cursor.tokens.get(cursor.at), Some(Token::LParen)) && !columns(cursor) {
            return false;
        }
        if !cursor.comma() {
            return true;
        }
    }
}
