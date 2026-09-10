//! PostgreSQL schema and embedded-SQL fact sources.
//!
//! Check rules consume these facts instead of re-parsing SQL or TypeScript.

mod annotation;
mod catalog;
mod collect;
mod conflict;
pub mod dml;
mod embedded;
mod idents;
mod locking;
mod migration;
mod offset;
mod on_conflict;
mod parse;
mod profiles;
mod rule_options;
mod schema;
mod statement_facts;
pub mod statements;
mod types;

pub use annotation::sql_requires_query_annotation;
pub(crate) use catalog::normalize_catalog_path as normalize_schema_catalog_path;
pub use catalog::{
    expression_matches, order_prefix_matches, parse_postgres_expression, CanonicalIndex,
    CanonicalOrderKey, ResolvedArbiter, SchemaCatalog,
};
pub use collect::{
    collect_postgres_facts, collect_schema_facts, extract_embedded_sql_facts, extract_schema_facts,
    postgres_sql_paths,
};
pub use conflict::{
    analyze_conflict_inserts, SqlConflictInsertFact, SqlConflictTarget, SqlInsertSourceShape,
};
pub use dml::{
    extract_dml_write_targets, find_generated_column_writes, GeneratedColumnWrite, GeneratedTable,
    GeneratedTableColumns,
};
pub(crate) use embedded::recovered_sql_needs_insert_check;
pub use embedded::{
    executed_query_text, executor_bindings, extract_embedded_sql_from_program,
    extract_embedded_sql_from_source, is_database_call, sql_text, EmbeddedSqlCall,
    EmbeddedSqlFileFacts, EmbeddedSqlKind, EmbeddedSqlOptions,
};
pub use locking::{extract_locking_select_metadata, LockingSelectMetadata};
pub use migration::extract_migration_facts;
pub use offset::sql_has_offset_clause;
pub use on_conflict::{judge_file, Catalog as IdempotentCatalog};
pub use parse::{parse_postgres_sql, PostgresParseError};
pub(crate) use profiles::{
    configured_embedded_sql_options, load_schema_catalogs, prepare_embedded_sql_facts,
};
pub use profiles::{
    configured_embedded_sql_options_for_checks, configured_schema_catalog_paths,
    PREPARED_EMBEDDED_SQL_RULE_IDS, SCHEMA_CATALOG_RULE_IDS,
};
pub use rule_options::fail_unanalyzable_sql;
pub use schema::extract_create_table_metadata;
pub use statements::{
    extract_sql_statement_facts, has_top_level_not_exists_in, insert_keyword_count,
    mask_quoted_sql, SqlAssignmentFact, SqlConflictArbiter, SqlConflictWhereProof,
    SqlExistsSetOpFact, SqlInsertFact, SqlOnConflictAction, SqlOnConflictFact, SqlSelectFact,
    SqlStatementFileFacts, SqlTriggerEvent, SqlTriggerFact, SqlTriggerPeriod, SqlValueForm,
};
pub use types::{
    PostgresFactError, PostgresFacts, PostgresSchemaOptions, SqlAddColumnMetadata,
    SqlColumnMetadata, SqlCreateIndexMetadata, SqlCreateTableMetadata, SqlDropIndexMetadata,
    SqlForeignKeyMetadata, SqlIndexParam, SqlNamedConstraint, SqlSchemaFileFacts, SqlStatementKind,
    SqlUnnamedConstraint,
};

#[cfg(test)]
mod tests;
