use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::{Parser, ParserError};
use std::fmt;

/// Parse failure for PostgreSQL SQL. Never panics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostgresParseError {
    pub message: String,
}

impl fmt::Display for PostgresParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for PostgresParseError {}

impl From<ParserError> for PostgresParseError {
    fn from(error: ParserError) -> Self {
        let message = match error {
            ParserError::ParserError(message) | ParserError::TokenizerError(message) => message,
            ParserError::RecursionLimitExceeded => error.to_string(),
        };
        Self { message }
    }
}

/// Parse `sql` with the PostgreSQL dialect.
pub fn parse_postgres_sql(sql: &str) -> Result<Vec<Statement>, PostgresParseError> {
    Parser::parse_sql(&PostgreSqlDialect {}, sql).map_err(PostgresParseError::from)
}

#[cfg(test)]
mod tests;
