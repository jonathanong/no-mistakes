//! PostgreSQL schema and embedded-SQL fact sources.
//!
//! Later check rules consume these facts instead of re-parsing SQL or
//! TypeScript. There is no CLI command and no check rule in this module.

mod collect;
mod embedded;
mod parse;
mod schema;
mod types;

pub use collect::{
    collect_postgres_facts, collect_schema_facts, extract_embedded_sql_facts, extract_schema_facts,
};
pub use embedded::{
    executed_query_text, executor_bindings, extract_embedded_sql_from_program,
    extract_embedded_sql_from_source, is_database_call, sql_text, EmbeddedSqlCall,
    EmbeddedSqlFileFacts, EmbeddedSqlOptions,
};
pub use parse::{parse_postgres_sql, PostgresParseError};
pub use schema::extract_create_table_metadata;
pub use types::{
    PostgresFactError, PostgresFacts, PostgresSchemaOptions, SqlColumnMetadata,
    SqlCreateTableMetadata, SqlSchemaFileFacts,
};

#[cfg(test)]
mod tests;
