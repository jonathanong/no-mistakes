//! Validate a complete PL/pgSQL routine wrapper before clearing its parse marker.
use super::{word, TokenWithSpan};
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::Token;

pub(super) fn complete_routine(code: &[&TokenWithSpan]) -> bool {
    let kind = if code.get(1).is_some_and(|token| word(token, "OR"))
        && code.get(2).is_some_and(|token| word(token, "REPLACE"))
    {
        3
    } else {
        1
    };
    let Some(token) = code.get(kind) else {
        return false;
    };
    if !word(token, "FUNCTION") && !word(token, "PROCEDURE") {
        return false;
    }
    // sqlparser's PostgreSQL FUNCTION grammar validates the full header/body and EOF.
    // Its PROCEDURE grammar expects procedural statements after AS, so normalize only
    // that keyword for this validation pass; retained facts use the original tokens.
    let mut tokens: Vec<_> = code.iter().map(|token| (*token).clone()).collect();
    if word(token, "PROCEDURE") {
        let Token::Word(procedure) = &mut tokens[kind].token else {
            return false;
        };
        procedure.value = "FUNCTION".to_owned();
        procedure.keyword = Keyword::FUNCTION;
    }
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(tokens);
    matches!(parser.parse_statement(), Ok(Statement::CreateFunction(_)))
        && parser.peek_token().token == Token::EOF
}
