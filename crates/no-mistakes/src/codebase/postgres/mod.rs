//! PostgreSQL schema and embedded-SQL fact sources.
//!
//! Check rules consume these facts instead of re-parsing SQL or TypeScript.

mod collect;
pub mod dml;
mod embedded;
mod locking;
mod migration;
mod offset;
mod parse;
mod schema;
mod types;

pub use collect::{
    collect_postgres_facts, collect_schema_facts, extract_embedded_sql_facts, extract_schema_facts,
};
pub use dml::{
    extract_dml_write_targets, find_generated_column_writes, GeneratedColumnWrite, GeneratedTable,
    GeneratedTableColumns,
};
pub use embedded::{
    executed_query_text, executor_bindings, extract_embedded_sql_from_program,
    extract_embedded_sql_from_source, is_database_call, sql_text, EmbeddedSqlCall,
    EmbeddedSqlFileFacts, EmbeddedSqlOptions,
};
pub use locking::{extract_locking_select_metadata, LockingSelectMetadata};
pub use migration::extract_migration_facts;
pub use offset::sql_has_offset_clause;
pub use parse::{parse_postgres_sql, PostgresParseError};
pub use schema::extract_create_table_metadata;
pub use types::{
    PostgresFactError, PostgresFacts, PostgresSchemaOptions, SqlColumnMetadata,
    SqlCreateIndexMetadata, SqlCreateTableMetadata, SqlDropIndexMetadata, SqlForeignKeyMetadata,
    SqlIndexParam, SqlNamedConstraint, SqlSchemaFileFacts,
};

#[cfg(test)]
mod tests;
