//! The native parser remains unchanged unless a prepared conflict marker is present.
use super::{parse_statement, parsing, PolicyArbiter, PostgreSqlDialect, SqlFunctionCallFact};
use sqlparser::{
    ast::Statement,
    keywords::Keyword,
    parser::{Parser, ParserError},
    tokenizer::{Location, Token, TokenWithSpan},
};

type ParsedPolicyProgram = (Vec<Statement>, Vec<SqlFunctionCallFact>, Vec<PolicyArbiter>);

pub(in crate::codebase::postgres) fn parse(
    mut tokens: Vec<TokenWithSpan>,
    mut locations: Option<&mut Vec<Location>>,
) -> Result<ParsedPolicyProgram, ParserError> {
    let prepared = parsing::prepare(&mut tokens, false);
    let markers = prepared.markers;
    // Markers already saw `BEGIN ATOMIC`. sqlparser must not nest the body.
    super::super::begin_atomic::conceal_body_keyword(&mut tokens);
    let dialect = PostgreSqlDialect {};
    let mut parser = Parser::new(&dialect).with_tokens_with_locations(tokens);
    if markers.is_empty() && locations.is_none() {
        return parser
            .parse_statements()
            .map(|statements| (statements, Vec::new(), Vec::new()));
    }
    let mut statements = Vec::new();
    let mut functions = Vec::new();
    let mut arbiters = Vec::new();
    let mut delimiter = false;
    loop {
        while parser.consume_token(&Token::SemiColon) {
            delimiter = false;
        }
        let token = parser.peek_token();
        if token.token == Token::EOF {
            break;
        }
        // Preserve the native statement-list boundary but do not certify a partial script.
        if delimiter && matches!(&token.token, Token::Word(word) if word.keyword == Keyword::END) {
            if let Some(locations) = &mut locations {
                locations.clear();
            }
            break;
        }
        if delimiter {
            return parser.expected_ref("end of statement", &token);
        }
        if let Some(locations) = &mut locations {
            locations.push(token.span.start);
        }
        if markers.is_empty() {
            statements.push(parser.parse_statement()?);
        } else {
            let (statement, mut calls, arbiter) =
                parse_statement(&mut parser, &markers, &prepared.cte_predicates)?;
            statements.push(statement);
            functions.append(&mut calls);
            arbiters.extend(arbiter);
        }
        delimiter = true;
    }
    Ok((statements, functions, arbiters))
}
