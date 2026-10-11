//! Reuse the source parser extension within the existing statement recovery boundary.
use super::insert::parsing;
use crate::codebase::postgres::{function_calls, SqlFunctionCallFact, SqlFunctionClause};
use sqlparser::{
    ast::{Expr, Statement},
    dialect::PostgreSqlDialect,
    parser::{Parser, ParserError},
    tokenizer::{Location, Token, TokenWithSpan},
};

/// Expression targets retained by the policy projection of the canonical parse.
#[derive(Debug)]
pub(crate) struct PolicyArbiter {
    pub insert_start: Location,
    pub expressions: Vec<Expr>,
    pub has_operator_class: bool,
}

pub(in crate::codebase::postgres) fn recover(
    tokens: &[TokenWithSpan],
) -> Option<(Statement, Vec<SqlFunctionCallFact>, Option<PolicyArbiter>)> {
    let mut tokens = tokens.to_vec();
    let markers = parsing::prepare(&mut tokens);
    if markers.is_empty() {
        return None;
    }
    let dialect = PostgreSqlDialect {};
    let mut parser = Parser::new(&dialect).with_tokens_with_locations(tokens);
    let (statement, functions, arbiter) = parse_statement(&mut parser, &markers).ok()?;
    (parser.peek_token().token == Token::EOF).then_some((statement, functions, arbiter))
}

pub(super) fn parse_statement(
    parser: &mut Parser<'_>,
    markers: &[Location],
) -> Result<(Statement, Vec<SqlFunctionCallFact>, Option<PolicyArbiter>), ParserError> {
    let (mut statement, facts) = parsing::parse_policy(parser, markers)?;
    let mut functions = Vec::new();
    let mut policy_arbiter = None;
    if let Some(facts) = facts {
        if !facts.expressions.is_empty() {
            let insert_start = parsing::inner_insert(&mut statement)
                .map(|insert| insert.insert_token.0.span.start);
            if let Some(insert_start) = insert_start {
                policy_arbiter = Some(PolicyArbiter {
                    insert_start,
                    has_operator_class: facts
                        .expressions
                        .iter()
                        .any(|arbiter| arbiter.operator_class.is_some()),
                    expressions: facts
                        .expressions
                        .iter()
                        .map(|arbiter| arbiter.expression.expression.clone())
                        .collect(),
                });
            }
        }
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
    Ok((statement, functions, policy_arbiter))
}

mod program;
pub(in crate::codebase::postgres) use program::parse as parse_program;

#[cfg(test)]
mod tests;
