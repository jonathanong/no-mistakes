use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::{Parser, ParserError};
use std::fmt;

mod lenient;
pub(super) mod unicode;
mod unicode_decode;

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

/// Parse `sql`, skipping unparseable statements instead of failing the file.
///
/// Migration trees mix parseable `CREATE TABLE` with `DO $$` blocks and other
/// statements sqlparser rejects. `DO $tag$` bodies are peeled and schema DDL
/// inside them is recovered, including `ALTER TABLE` after PL/pgSQL `IF/THEN`.
/// Other unparseable SQL is still skipped. PostgreSQL 18
/// `GENERATED ALWAYS AS (...) VIRTUAL` is rewritten to `STORED` so those
/// `CREATE TABLE` statements parse.
pub fn parse_postgres_sql_lenient(sql: &str) -> Vec<Statement> {
    lenient::parse_postgres_sql_lenient(sql)
}

pub(crate) fn expand_chr_encoded_sql(sql: &str) -> Option<String> {
    lenient::expand_chr_encoded_sql(sql)
}

#[cfg(test)]
mod tests;
