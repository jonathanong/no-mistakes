use sqlparser::ast::Statement;
use sqlparser::parser::ParserError;
use std::fmt;

mod copy_data;
mod ddl_compat;
mod derived_table;
mod distinct_group;
pub(super) mod fetch_expression;
mod function_sources;
pub(crate) use function_sources::orphan_chunk_tokens;
#[doc(hidden)]
pub use function_sources::partition as partition_function_sources;
mod lenient;
pub(crate) use lenient::extension_has_version;
mod lock_of_list;
mod lock_strength;
mod named_arguments;
#[doc(hidden)]
pub mod operator_boundary;
mod original_head;
mod prepared;
pub(crate) use lenient::parse_postgres_body_with_function_sources;
#[doc(hidden)]
pub use lenient::parse_postgres_sql_with_function_sources;
pub(crate) use lenient::rewrite::rewrite_referential_set_column_lists_located;
#[doc(hidden)]
pub use lenient::LocatedStatement;
pub(crate) use lenient::ProceduralBodyState;
#[doc(hidden)]
pub use prepared::PreparedSql;
mod radix_numbers;
mod recursive_view;
pub(crate) use recursive_view::RecursiveViews;
mod source_escape;
mod source_unicode;
mod sql_text;
mod standalone_table;
mod statement_head;
mod table_boundary;
mod table_only;
use sql_text::located_statements;
pub(super) use sql_text::{normalize_copy_data, normalize_copy_data_for_tokens};
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
    PreparedSql::new(sql).parse()
}

/// Parse a standalone PostgreSQL expression via `SELECT <expression>`.
pub fn parse_postgres_expression(expression: &str) -> Option<sqlparser::ast::Expr> {
    let mut statements = parse_postgres_sql(&format!("SELECT {expression}")).ok()?;
    let Statement::Query(query) = statements.pop()? else {
        return None;
    };
    let sqlparser::ast::SetExpr::Select(select) = *query.body else {
        return None;
    };
    (select.projection.len() == 1).then_some(())?;
    match select.projection.into_iter().next()? {
        sqlparser::ast::SelectItem::UnnamedExpr(expression)
        | sqlparser::ast::SelectItem::ExprWithAlias {
            expr: expression, ..
        } => Some(expression),
        _ => None,
    }
}

pub(super) struct PreparedPostgresTokens {
    pub tokens: Vec<sqlparser::tokenizer::TokenWithSpan>,
    /// Pre-rewrite lexical tokens for procedural occurrence classification.
    /// Ordinary preparation leaves this empty. A procedural walk retains the
    /// inventory so `FOR UPDATE OF a, b` is not expanded into duplicate DML facts.
    pub occurrence_tokens: Vec<sqlparser::tokenizer::TokenWithSpan>,
    pub recursive_views: RecursiveViews,
    pub lexical_error: Option<sqlparser::tokenizer::TokenizerError>,
}

/// Prepare one source-fact token inventory while retaining a valid lexical prefix.
pub(super) fn prepare_postgres_tokens(sql: &str) -> PreparedPostgresTokens {
    prepare_postgres_tokens_inner(sql, false)
}

/// Prepare tokens and retain the pre-rewrite inventory for a procedural walk.
pub(crate) fn prepare_postgres_tokens_for_walk(sql: &str) -> PreparedPostgresTokens {
    prepare_postgres_tokens_inner(sql, true)
}

fn prepare_postgres_tokens_inner(
    sql: &str,
    retain_occurrence_tokens: bool,
) -> PreparedPostgresTokens {
    let normalized = normalize_copy_data(sql);
    let separated = distinct_group::separate_distinct_grouping(&normalized);
    let escaped = source_escape::prepare(&separated);
    let boundary = operator_boundary::Prepared::new(&escaped.sql);
    let mut tokens = Vec::new();
    let mut lexical_error =
        sqlparser::tokenizer::Tokenizer::new(&source_unicode::SourceDialect, boundary.sql())
            .tokenize_with_location_into_buf(&mut tokens)
            .err();
    boundary.restore(&mut tokens);
    if let Some(error) = &mut lexical_error {
        boundary.restore_error(error);
    }
    escaped.restore(&mut tokens);
    source_unicode::prepare(&mut tokens);
    let mut tokens = radix_numbers::repair(&tokens).unwrap_or(tokens);
    let occurrence_tokens = if retain_occurrence_tokens {
        tokens.clone()
    } else {
        Vec::new()
    };
    normalize_table_queries(&mut tokens);
    let recursive_views = recursive_view::prepare(&mut tokens);
    PreparedPostgresTokens {
        tokens,
        occurrence_tokens,
        recursive_views,
        lexical_error,
    }
}

fn normalize_table_queries(tokens: &mut Vec<sqlparser::tokenizer::TokenWithSpan>) {
    table_only::normalize(tokens);
    derived_table::normalize(tokens);
    standalone_table::normalize(tokens);
    table_boundary::normalize(tokens);
    lock_strength::normalize(tokens);
    lock_of_list::normalize(tokens);
}

/// Parse `sql`, skipping unparseable statements instead of failing the file.
/// Peels `DO $tag$` bodies, rewrites PostgreSQL 18 `GENERATED ... VIRTUAL` to
/// `STORED`, and strips `ON DELETE SET NULL/DEFAULT` column lists before parse.
pub fn parse_postgres_sql_lenient(sql: &str) -> Vec<Statement> {
    lenient::parse_postgres_sql_lenient(sql)
}

#[doc(hidden)]
pub fn parse_postgres_sql_lenient_with_sources(
    sql: &str,
    normalized: &str,
) -> Vec<LocatedStatement> {
    lenient::parse_postgres_sql_lenient_with_sources(sql, normalized)
}

pub(crate) fn expand_chr_encoded_sql(sql: &str) -> Option<String> {
    lenient::expand_chr_encoded_sql(sql)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod analyze_list_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod expression_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod radix_first_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod radix_large_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod radix_lenient_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod recovered_hex_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod uppercase_hex_tests;
