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
    // The parser below validates CREATE, the complete header/body, and EOF.
    // Only the unsupported PROCEDURE keyword is normalized for this proof.
    let mut tokens: Vec<_> = code.iter().map(|token| (*token).clone()).collect();
    if let Some(TokenWithSpan {
        token: Token::Word(procedure),
        ..
    }) = tokens.get_mut(kind)
    {
        if procedure.quote_style.is_none() && procedure.value.eq_ignore_ascii_case("PROCEDURE") {
            procedure.value = "FUNCTION".to_owned();
            procedure.keyword = Keyword::FUNCTION;
        }
    }
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(tokens);
    matches!(parser.parse_statement(), Ok(Statement::CreateFunction(_)))
        && parser.peek_token().token == Token::EOF
}
