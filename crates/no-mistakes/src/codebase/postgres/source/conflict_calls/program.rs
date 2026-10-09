//! The native parser remains unchanged unless a prepared conflict marker is present.
use super::{parse_statement, parsing, PostgreSqlDialect, SqlFunctionCallFact};
use sqlparser::{
    ast::Statement,
    keywords::Keyword,
    parser::{Parser, ParserError},
    tokenizer::{Token, TokenWithSpan},
};

pub(in crate::codebase::postgres) fn parse(
    mut tokens: Vec<TokenWithSpan>,
) -> Result<(Vec<Statement>, Vec<SqlFunctionCallFact>), ParserError> {
    let markers = parsing::prepare(&mut tokens);
    let dialect = PostgreSqlDialect {};
    let mut parser = Parser::new(&dialect).with_tokens_with_locations(tokens);
    if markers.is_empty() {
        return parser
            .parse_statements()
            .map(|statements| (statements, Vec::new()));
    }
    let mut statements = Vec::new();
    let mut functions = Vec::new();
    let mut delimiter = false;
    loop {
        while parser.consume_token(&Token::SemiColon) {
            delimiter = false;
        }
        let token = parser.peek_token();
        // Match the native statement-list boundary, including an enclosing END.
        if token.token == Token::EOF
            || delimiter
                && matches!(&token.token, Token::Word(word) if word.keyword == Keyword::END)
        {
            break;
        }
        if delimiter {
            return parser.expected_ref("end of statement", &token);
        }
        let (statement, mut calls) = parse_statement(&mut parser, &markers)?;
        statements.push(statement);
        functions.append(&mut calls);
        delimiter = true;
    }
    Ok((statements, functions))
}
