use super::Cursor;
use sqlparser::tokenizer::Token;

pub(super) fn valid(cursor: &mut Cursor<'_>) -> bool {
    if !cursor.identifier() {
        return false;
    }
    if cursor.word("FOR")
        && !(cursor.words(&["ALL", "TABLES"]) || cursor.word("TABLE") && cursor.names())
    {
        return false;
    }
    if !cursor.word("WITH") {
        return true;
    }
    options(cursor)
}

pub(super) fn alter(cursor: &mut Cursor<'_>) -> bool {
    if !cursor.identifier() {
        return false;
    }
    if cursor.word("RENAME") {
        return cursor.word("TO") && cursor.identifier();
    }
    if cursor.word("OWNER") {
        return cursor.word("TO")
            && (cursor.word("CURRENT_ROLE")
                || cursor.word("CURRENT_USER")
                || cursor.word("SESSION_USER")
                || cursor.identifier());
    }
    if cursor.word("SET") {
        if cursor
            .tokens
            .get(cursor.at)
            .is_some_and(|token| **token == Token::LParen)
        {
            return options(cursor);
        }
    } else if !(cursor.word("ADD") || cursor.word("DROP")) {
        return false;
    }
    cursor.word("TABLE") && cursor.names()
}

pub(super) fn drop(cursor: &mut Cursor<'_>) -> bool {
    if cursor.word("IF") && !cursor.word("EXISTS") {
        return false;
    }
    cursor.identifiers() && {
        if !cursor.word("CASCADE") {
            cursor.word("RESTRICT");
        }
        true
    }
}

fn options(cursor: &mut Cursor<'_>) -> bool {
    if !take(cursor, &Token::LParen) {
        return false;
    }
    loop {
        if !cursor.identifier() || !take(cursor, &Token::Eq) || !value(cursor) {
            return false;
        }
        if !cursor.comma() {
            return take(cursor, &Token::RParen);
        }
    }
}

fn value(cursor: &mut Cursor<'_>) -> bool {
    // PostgreSQL publication parameters use scalar option values, never expressions.
    if matches!(
        cursor.tokens.get(cursor.at),
        Some(Token::SingleQuotedString(_) | Token::Number(_, _))
    ) {
        cursor.at += 1;
        true
    } else {
        cursor.word("TRUE") || cursor.word("FALSE") || cursor.identifier()
    }
}

fn take(cursor: &mut Cursor<'_>, token: &Token) -> bool {
    if cursor
        .tokens
        .get(cursor.at)
        .is_some_and(|actual| *actual == token)
    {
        cursor.at += 1;
        true
    } else {
        false
    }
}
