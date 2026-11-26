//! PostgreSQL SQL parsing, statement facts, and identifier extraction.

mod fx;
#[cfg(test)]
pub(crate) mod test_support;

pub mod annotation;
pub mod bound_expr;
pub mod catalog_hooks;
pub mod conflict;
pub mod dml;
pub mod function_calls;
pub mod idents;
pub mod lock_inventory;
pub mod locking;
pub mod migration;
pub mod migration_order;
pub mod numeric_literal;
pub mod offset;
pub mod order;
pub mod parse;
pub mod predicate_normalization;
pub mod schema;
pub mod source;
pub mod sql_include_policy;
pub mod sql_names;
pub mod statement_facts;
pub mod statements;
pub mod types;

pub use annotation::{leading_query_annotation, sql_requires_query_annotation};
pub use bound_expr::canonical_bound_expr;
pub use catalog_hooks::{
    CatalogLifecycleLookup, CatalogTriggerWrite, SearchPathResolution, TriggerEvent,
};
pub use conflict::{
    analyze_conflict_inserts, analyze_conflict_inserts_with_binds, expression_is_constant,
    SqlConflictInsertFact, SqlConflictTarget, SqlInsertSourceShape, SqlPinnedJoinEquality,
    SqlPinnedRelation, SqlPinnedSource, SqlSourceRelation,
};
pub use dml::{
    extract_dml_write_targets, find_generated_column_writes, GeneratedColumnWrite, GeneratedTable,
    GeneratedTableColumns,
};
pub use function_calls::{SqlFunctionCallFact, SqlFunctionClause};
pub use locking::{
    extract_locking_select_metadata, extract_locking_select_metadata_with_placeholders,
    JoinEquality, LockingSelectMetadata,
};
pub use migration::extract_migration_facts;
pub use offset::{
    sql_file_offset_uses, sql_has_offset_clause, sql_offset_uses, OffsetUse, SqlOffsetFact,
};
pub use order::{canonical_order_keys, order_by_ascending, CanonicalOrderKey};
pub use parse::{parse_postgres_expression, parse_postgres_sql, PostgresParseError};
pub use schema::extract_create_table_metadata;
pub use source::*;
pub use sql_names::decoded_parts;
pub use statement_facts::SqlLifecycleFacts;
#[doc(hidden)]
pub use statement_facts::{SqlLifecycleBatch, SqlLifecycleStep, SqlViewReads};
pub use statements::{
    extract_sql_statement_facts, has_top_level_not_exists_in, insert_keyword_count,
    mask_quoted_sql, SqlAssignmentFact, SqlBareRead, SqlBoundFact, SqlBoundInputMode, SqlBoundItem,
    SqlBoundItemKind, SqlBoundKind, SqlBoundOutput, SqlBoundPin, SqlBoundQuery, SqlColumnClause,
    SqlColumnUseFact, SqlConflictArbiter, SqlConflictWhereProof, SqlConjunctFact,
    SqlCountExistenceFact, SqlCursorBound, SqlExistsSetOpFact, SqlInsertFact, SqlLimitFact,
    SqlLimitValue, SqlOnConflictAction, SqlOnConflictFact, SqlPinSource, SqlQualifiedRead,
    SqlQualifiedScope, SqlRelationPredicateFact, SqlScalarReads, SqlSelectFact,
    SqlStatementFileFacts, SqlSweepFact, SqlTriggerEvent, SqlTriggerFact, SqlTriggerPeriod,
    SqlValueForm, SqlWriteColumns, SqlWriteFact,
};
pub use types::{
    PostgresFactError, PostgresSchemaOptions, SqlAddColumnMetadata, SqlColumnMetadata,
    SqlCreateIndexMetadata, SqlCreateTableMetadata, SqlDeclaredIdentifier, SqlDropIndexMetadata,
    SqlForeignKeyMetadata, SqlIndexParam, SqlNamedConstraint, SqlSchemaFileFacts, SqlSettingUse,
    SqlStatementKind, SqlTableSchemaEvent, SqlUnanalyzableStatement, SqlUnnamedConstraint,
};
#[doc(hidden)]
pub use types::{SqlPolicyClause, SqlPolicyClauseUse};
