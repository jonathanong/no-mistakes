use super::{
    SqlAddColumnMetadata, SqlCreateIndexMetadata, SqlDropIndexMetadata, SqlForeignKeyMetadata,
    SqlNamedConstraint, SqlSettingUse, SqlStatementKind, SqlTableSchemaEvent, SqlUnnamedConstraint,
};
use std::path::PathBuf;

/// One `CREATE TABLE` after PostgreSQL dialect parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlCreateTableMetadata {
    pub table_name: String,
    pub columns: Vec<SqlColumnMetadata>,
}

/// Column facts later generated-column and lock-ordering rules can query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlColumnMetadata {
    pub name: String,
    pub type_name: Option<String>,
    pub constraints: Vec<String>,
    pub is_primary_key: bool,
    pub is_generated: bool,
    pub generated_expression: Option<String>,
    pub generated_function: Option<String>,
    pub generated_function_arg_columns: Vec<String>,
    pub generated_source_columns: Vec<String>,
}

/// A name introduced by one migration statement, unquoted and schema-unqualified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlDeclaredIdentifier {
    pub name: String,
    pub line: usize,
}

/// A semicolon-delimited SQL chunk that lenient parsing could not recover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlUnanalyzableStatement {
    pub line: usize,
    pub head: String,
    /// Lexical statement ordinals, including enclosing routine bodies.
    pub source_order: Vec<usize>,
}

/// Schema facts for one SQL file read through [`crate::codebase::ts_source::SourceStore`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SqlSchemaFileFacts {
    pub path: PathBuf,
    pub unanalyzable_statements: Vec<SqlUnanalyzableStatement>,
    pub tables: Vec<SqlCreateTableMetadata>,
    /// Qualified CREATE/ALTER/DROP operations in statement order for live-column catalogs.
    pub table_events: Vec<SqlTableSchemaEvent>,
    /// Distinguishes an empty execution projection from legacy hand-built facts.
    pub table_events_collected: bool,
    pub indexes: Vec<SqlCreateIndexMetadata>,
    pub dropped_indexes: Vec<SqlDropIndexMetadata>,
    pub dropped_tables: Vec<SqlDropIndexMetadata>,
    pub foreign_keys: Vec<SqlForeignKeyMetadata>,
    pub add_columns: Vec<SqlAddColumnMetadata>,
    pub unnamed_constraints: Vec<SqlUnnamedConstraint>,
    pub statement_kinds: Vec<SqlStatementKind>,
    pub setting_uses: Vec<SqlSettingUse>,
    pub function_calls: Vec<super::super::SqlFunctionCallFact>,
    pub not_valid_constraints: Vec<SqlNamedConstraint>,
    pub validated_constraints: Vec<SqlNamedConstraint>,
    pub declared_identifiers: Vec<SqlDeclaredIdentifier>,
}
