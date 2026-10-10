use crate::codebase::postgres::source::types::PostgresSqlProceduralOccurrence;
use sqlparser::{
    dialect::PostgreSqlDialect,
    parser::Parser,
    tokenizer::{Token, TokenWithSpan},
};

pub(super) fn statements_are_sql(
    tokens: &[TokenWithSpan],
    occurrences: &[PostgresSqlProceduralOccurrence],
) -> bool {
    occurrences
        .iter()
        .all(|occurrence| statement_parses(tokens, occurrence))
        && command_is_covered(tokens, occurrences)
}

fn command_is_covered(
    tokens: &[TokenWithSpan],
    occurrences: &[PostgresSqlProceduralOccurrence],
) -> bool {
    let mut covered = vec![false; tokens.len()];
    for occurrence in occurrences {
        let start = occurrence.span.start.offset;
        let end = occurrence.span.end.offset;
        if start > end || end >= tokens.len() {
            return false;
        }
        for slot in &mut covered[start..=end] {
            *slot = true;
        }
    }
    tokens.iter().enumerate().all(|(index, token)| {
        covered[index] || matches!(token.token, Token::Whitespace(_) | Token::SemiColon)
    })
}

fn statement_parses(
    tokens: &[TokenWithSpan],
    occurrence: &PostgresSqlProceduralOccurrence,
) -> bool {
    let start = occurrence.span.start.offset;
    let end = occurrence.span.end.offset;
    if start > end || end >= tokens.len() {
        return false;
    }
    let mut statement = tokens[start..=end]
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)))
        .cloned()
        .collect::<Vec<_>>();
    // The span includes the statement terminator. A semicolon inside the
    // statement is still syntax, so only the final one is dropped.
    if matches!(
        statement.last().map(|token| &token.token),
        Some(Token::SemiColon)
    ) {
        statement.pop();
    }
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(statement);
    parser.parse_statement().is_ok() && parser.peek_token().token == Token::EOF
}
