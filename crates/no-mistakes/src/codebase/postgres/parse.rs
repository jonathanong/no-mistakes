use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::{Parser, ParserError};
use std::fmt;

mod copy_data;
mod distinct_group;
mod lenient;
pub(crate) use lenient::LocatedStatement;
mod radix_numbers;
mod sql_text;
mod standalone_table;
mod table_only;
pub(super) use sql_text::normalize_copy_data;
pub(crate) use sql_text::top_level_statements;
pub(super) mod unicode;
mod unicode_decode;

/// Match the built-in language using PostgreSQL identifier casing.
pub(super) fn is_plpgsql_language(token: &sqlparser::tokenizer::Token) -> bool {
    matches!(token, sqlparser::tokenizer::Token::Word(language)
        if language.value == "plpgsql"
            || language.quote_style.is_none() && language.value.eq_ignore_ascii_case("plpgsql"))
}

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
    let normalized = normalize_copy_data(sql);
    let separated = distinct_group::separate_distinct_grouping(&normalized);
    let tokens = sqlparser::tokenizer::Tokenizer::new(&PostgreSqlDialect {}, &separated)
        .tokenize_with_location()
        .map_err(|error| PostgresParseError {
            message: error.to_string(),
        })?;
    let mut tokens = radix_numbers::repair(&tokens).unwrap_or(tokens);
    table_only::normalize(&mut tokens);
    standalone_table::normalize(&mut tokens);
    Parser::new(&PostgreSqlDialect {})
        .with_tokens_with_locations(tokens)
        .parse_statements()
        .map_err(PostgresParseError::from)
}

/// Parse `sql`, skipping unparseable statements instead of failing the file.
///
/// Migration trees mix parseable `CREATE TABLE` with `DO $$` blocks and other
/// statements sqlparser rejects. `DO $tag$` bodies are peeled and schema DDL
/// inside them is recovered, including schema and DML statements after PL/pgSQL
/// `BEGIN` or `IF/THEN` prefixes.
/// Other unparseable SQL is still skipped. PostgreSQL 18
/// `GENERATED ALWAYS AS (...) VIRTUAL` is rewritten to `STORED` so those
/// `CREATE TABLE` statements parse. Column lists on `ON DELETE SET NULL` /
/// `SET DEFAULT` are removed before parsing; the action keyword and
/// constraint name stay. `ON UPDATE` column lists are left unchanged.
pub fn parse_postgres_sql_lenient(sql: &str) -> Vec<Statement> {
    lenient::parse_postgres_sql_lenient(sql)
}

pub(crate) fn parse_postgres_sql_lenient_with_sources(sql: &str) -> Vec<LocatedStatement> {
    lenient::parse_postgres_sql_lenient_with_sources(sql)
}

pub(crate) fn expand_chr_encoded_sql(sql: &str) -> Option<String> {
    lenient::expand_chr_encoded_sql(sql)
}

#[cfg(test)]
mod radix_first_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod radix_lenient_tests;

#[cfg(test)]
mod radix_large_tests;

#[cfg(test)]
mod recovered_hex_tests;

#[cfg(test)]
mod uppercase_hex_tests;
