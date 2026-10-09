//! Reuse the source parser extension within the existing statement recovery boundary.
use super::insert::parsing;
use crate::codebase::postgres::{function_calls, SqlFunctionCallFact, SqlFunctionClause};
use sqlparser::{
    ast::Statement,
    dialect::PostgreSqlDialect,
    parser::{Parser, ParserError},
    tokenizer::{Location, Token, TokenWithSpan},
};

pub(in crate::codebase::postgres) fn recover(
    tokens: &[TokenWithSpan],
) -> Option<(Statement, Vec<SqlFunctionCallFact>)> {
    let mut tokens = tokens.to_vec();
    let markers = parsing::prepare(&mut tokens);
    if markers.is_empty() {
        return None;
    }
    let dialect = PostgreSqlDialect {};
    let mut parser = Parser::new(&dialect).with_tokens_with_locations(tokens);
    let (statement, functions) = parse_statement(&mut parser, &markers).ok()?;
    (parser.peek_token().token == Token::EOF).then_some((statement, functions))
}

pub(super) fn parse_statement(
    parser: &mut Parser<'_>,
    markers: &[Location],
) -> Result<(Statement, Vec<SqlFunctionCallFact>), ParserError> {
    let (statement, facts) = parsing::parse_policy(parser, markers)?;
    let mut functions = Vec::new();
    if let Some(facts) = facts {
        for arbiter in &facts.expressions {
            function_calls::collect_expression(
                &arbiter.expression.expression,
                &mut functions,
                None,
            );
        }
        if let Some(predicate) = facts.predicate {
            function_calls::collect_expression(
                &predicate.expression,
                &mut functions,
                Some(SqlFunctionClause::Where),
            );
        }
    }
    Ok((statement, functions))
}

mod program;
pub(in crate::codebase::postgres) use program::parse as parse_program;

#[cfg(test)]
mod tests;
