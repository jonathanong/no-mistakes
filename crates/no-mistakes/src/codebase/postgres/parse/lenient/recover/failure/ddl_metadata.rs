//! Complete, deliberately narrow PostgreSQL 17 DDL grammars missing in sqlparser.
//! These remain unsupported statements, rather than fabricated schema facts.
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan};
mod catalog;
mod create;
mod cursor;
mod data_type;
mod definitions;
mod event;
use cursor::Cursor;

pub(super) fn valid(tokens: &[TokenWithSpan]) -> bool {
    let mut code: Vec<_> = tokens
        .iter()
        .filter(|t| !matches!(t.token, Token::Whitespace(_) | Token::EOF))
        .cloned()
        .collect();
    if code.last().is_some_and(|t| t.token == Token::SemiColon) {
        code.pop();
    }
    let parse = |f: fn(&mut Cursor<'_>) -> bool| {
        let mut cursor =
            Cursor(Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(code.clone()));
        f(&mut cursor) && cursor.0.peek_token().token == Token::EOF
    };
    parse(event::parse)
        || parse(definitions::parse)
        || parse(catalog::parse)
        || parse(create::parse)
}

#[cfg(test)]
mod tests;
