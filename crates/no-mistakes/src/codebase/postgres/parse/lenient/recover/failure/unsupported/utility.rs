//! Complete utility statements which do not contribute schema facts.
use super::cursor::Cursor;
use sqlparser::tokenizer::{Token, TokenWithSpan};
mod declaration;
mod maintenance;
mod privileges;
mod session;
#[cfg(test)]
#[path = "utility/tests.rs"]
mod tests;
mod transfer;

pub(super) fn valid(original: &[TokenWithSpan]) -> bool {
    let tokens: Vec<_> = original
        .iter()
        .map(|item| &item.token)
        .filter(|token| !matches!(token, Token::Whitespace(_)))
        .collect();
    [
        maintenance::valid,
        declaration::valid,
        privileges::valid,
        session::valid,
        transfer::valid,
    ]
    .into_iter()
    .any(|check| {
        let mut cursor = Cursor {
            tokens: &tokens,
            at: 0,
        };
        check(&mut cursor) && cursor.at == tokens.len()
    })
}

fn token(cursor: &mut Cursor<'_>, expected: Token) -> bool {
    if cursor
        .tokens
        .get(cursor.at)
        .is_some_and(|value| **value == expected)
    {
        cursor.at += 1;
        true
    } else {
        false
    }
}

fn number(cursor: &mut Cursor<'_>, signed: bool) -> bool {
    let start = cursor.at;
    if signed && !token(cursor, Token::Minus) {
        token(cursor, Token::Plus);
    }
    if matches!(cursor.tokens.get(cursor.at), Some(Token::Number(value, false))
        if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
    {
        cursor.at += 1;
        true
    } else {
        cursor.at = start;
        false
    }
}

fn boolean(cursor: &mut Cursor<'_>) -> bool {
    ["TRUE", "FALSE", "ON", "OFF"]
        .iter()
        .any(|value| cursor.word(value))
        || if matches!(cursor.tokens.get(cursor.at), Some(Token::Number(value, false)) if value == "0" || value == "1")
        {
            cursor.at += 1;
            true
        } else {
            false
        }
}

fn string(cursor: &mut Cursor<'_>) -> bool {
    if matches!(
        cursor.tokens.get(cursor.at),
        Some(Token::SingleQuotedString(_) | Token::EscapedStringLiteral(_))
    ) {
        cursor.at += 1;
        true
    } else {
        false
    }
}

fn columns(cursor: &mut Cursor<'_>) -> bool {
    token(cursor, Token::LParen) && cursor.identifiers() && token(cursor, Token::RParen)
}

fn expression(tokens: &[&Token]) -> bool {
    let input = tokens.iter().map(|token| (*token).clone()).collect();
    let mut parser = sqlparser::parser::Parser::new(&sqlparser::dialect::PostgreSqlDialect {})
        .with_tokens(input);
    parser
        .parse_expr()
        .is_ok_and(|expr| super::query::valid_ast(&expr))
        && parser.peek_token().token == Token::EOF
}
