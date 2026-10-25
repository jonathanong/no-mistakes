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
