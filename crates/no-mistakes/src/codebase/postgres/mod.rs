//! PostgreSQL schema and embedded-SQL fact sources.
//!
//! Check rules consume these facts instead of re-parsing SQL or TypeScript.

mod annotation;
mod catalog;
mod collect;
mod conflict;
pub mod dml;
mod embedded;
pub(crate) mod idents;
mod locking;
mod migration;
mod migration_order;
pub(crate) use migration_order::cmp_sql_rel;
mod numeric_literal;
mod offset;
mod on_conflict;
mod parse;
pub(crate) mod predicate_normalization;
pub(crate) mod prepared;
mod profiles;
mod rule_options;
mod schema;
mod source;
mod statement_facts;
pub use statement_facts::SqlLifecycleFacts;
pub(crate) use statement_facts::{SqlLifecycleBatch, SqlLifecycleStep, SqlViewReads};
pub(crate) use statements::project_bounds as project_sql_bounds;
pub mod statements;
mod types;

pub use annotation::sql_requires_query_annotation;
pub(crate) use catalog::canonical_order_keys;
pub(crate) use catalog::decoded_parts;
pub(crate) use catalog::normalize_catalog_path as normalize_schema_catalog_path;
pub(crate) use catalog::order_by_ascending;
pub(crate) use catalog::SearchPathResolution;
pub use catalog::{
    catalog_finding, expression_matches, order_prefix_matches, parse_postgres_expression,
    require_catalog_path, AllowEntry, AllowList, CanonicalIndex, CanonicalOrderKey, CatalogCheck,
    CatalogColumn, CatalogCoverage, CatalogEnum, CatalogForeignKey, CatalogFunction,
    CatalogIndexInfo, CatalogIndexKey, CatalogObjectRef, CatalogTable, CatalogTrigger,
    CatalogUnique, CatalogView, GeneratedKind, PartitionKey, PartitionKeyElement,
    PartitionStrategy, RelationKind, ResolvedArbiter, SchemaCatalog, TriggerEvent, TriggerTiming,
};
pub use collect::{
    collect_postgres_facts, collect_schema_facts, extract_embedded_sql_facts, extract_schema_facts,
    postgres_sql_paths,
};
pub use conflict::{
    analyze_conflict_inserts, analyze_conflict_inserts_with_binds, expression_is_constant,
    SqlConflictInsertFact, SqlConflictTarget, SqlInsertSourceShape, SqlPinnedRelation,
    SqlSourceRelation,
};
pub use dml::{
    extract_dml_write_targets, find_generated_column_writes, GeneratedColumnWrite, GeneratedTable,
    GeneratedTableColumns,
};
pub(crate) use embedded::recovered_sql_needs_insert_check;
pub use embedded::{
    executed_query_text, executor_bindings, extract_embedded_sql_from_program,
    extract_embedded_sql_from_source, is_database_call, sql_text, EmbeddedSqlCall,
    EmbeddedSqlFileFacts, EmbeddedSqlFragment, EmbeddedSqlKind, EmbeddedSqlOptions,
    EmbeddedSqlSourcePosition, TrustedSqlTag,
};
pub(crate) use embedded::{
    package_name, package_root_for_specifier, project_relative_scoped_facts,
};
pub use locking::{
    extract_locking_select_metadata, extract_locking_select_metadata_with_placeholders,
    JoinEquality, LockingSelectMetadata,
};
pub use migration::extract_migration_facts;
pub(crate) use migration::SUPPORTED_KINDS;
pub use offset::{
    sql_file_offset_uses, sql_has_offset_clause, sql_offset_uses, OffsetUse, SqlOffsetFact,
};
pub use on_conflict::{judge_file, Catalog as IdempotentCatalog};
pub use parse::{parse_postgres_sql, PostgresParseError};
pub use profiles::{
    configure_prepared_postgres_plan, configured_embedded_sql_options_for_checks,
    configured_schema_catalog_paths, PREPARED_EMBEDDED_SQL_RULE_IDS, SCHEMA_CATALOG_RULE_IDS,
};
pub(crate) use profiles::{
    configured_embedded_sql_options, load_schema_catalogs, prepare_embedded_sql_facts,
    prepare_rule_sql_facts,
};
pub use rule_options::fail_unanalyzable_sql;
pub use schema::extract_create_table_metadata;
pub use source::*;
pub use statements::{
    extract_sql_statement_facts, extract_sql_statement_facts_for_embedded_call,
    has_top_level_not_exists_in, insert_keyword_count, mask_quoted_sql, SqlAssignmentFact,
    SqlBareRead, SqlBoundFact, SqlBoundInputMode, SqlBoundItem, SqlBoundItemKind, SqlBoundKind,
    SqlBoundOutput, SqlBoundPin, SqlBoundQuery, SqlColumnClause, SqlColumnUseFact,
    SqlConflictArbiter, SqlConflictWhereProof, SqlConjunctFact, SqlCountExistenceFact,
    SqlCursorBound, SqlExistsSetOpFact, SqlInsertFact, SqlLimitFact, SqlLimitValue,
    SqlOnConflictAction, SqlOnConflictFact, SqlPinSource, SqlQualifiedRead, SqlQualifiedScope,
    SqlRelationPredicateFact, SqlSelectFact, SqlStatementFileFacts, SqlSweepFact, SqlTriggerEvent,
    SqlTriggerFact, SqlTriggerPeriod, SqlValueForm, SqlWriteColumns, SqlWriteFact,
};
pub use types::{
    PostgresFactError, PostgresFacts, PostgresSchemaOptions, SqlAddColumnMetadata,
    SqlColumnMetadata, SqlCreateIndexMetadata, SqlCreateTableMetadata, SqlDeclaredIdentifier,
    SqlDropIndexMetadata, SqlForeignKeyMetadata, SqlIndexParam, SqlNamedConstraint,
    SqlSchemaFileFacts, SqlSettingUse, SqlStatementKind, SqlTableSchemaEvent, SqlUnnamedConstraint,
};

#[cfg(test)]
pub(crate) mod tests;

pub(crate) use collect::collect_prepared_schema_facts;
