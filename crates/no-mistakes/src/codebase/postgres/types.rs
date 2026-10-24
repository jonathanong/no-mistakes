mod table_events;
pub use table_events::SqlTableSchemaEvent;
mod schema_file;
pub use schema_file::{
    SqlColumnMetadata, SqlCreateTableMetadata, SqlDeclaredIdentifier, SqlSchemaFileFacts,
    SqlUnanalyzableStatement,
};

use super::embedded::EmbeddedSqlFileFacts;
use std::fmt;
use std::path::{Path, PathBuf};

/// Options for selecting SQL schema files. There is no hardcoded migrations root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostgresSchemaOptions {
    pub sql_include: Vec<String>,
}

impl Default for PostgresSchemaOptions {
    fn default() -> Self {
        Self {
            sql_include: vec!["**/*.sql".to_string()],
        }
    }
}

/// One key column of a btree index, including opclass and sort options.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SqlIndexParam {
    pub name: Option<String>,
    pub expression: String,
    pub opclass: Option<String>,
    pub ordering: Option<String>,
    pub nulls_ordering: Option<String>,
}

/// One `DROP INDEX` so later migrations can remove earlier creates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlDropIndexMetadata {
    pub name: String,
    pub line: usize,
}

/// One `CREATE INDEX` or unique/primary-key covering index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlCreateIndexMetadata {
    pub table_name: String,
    pub name: Option<String>,
    pub leading_column: Option<String>,
    pub columns: Vec<SqlIndexParam>,
    pub include_columns: Vec<String>,
    pub access_method: String,
    pub unique: bool,
    pub has_predicate: bool,
    pub not_null_predicate_column: Option<String>,
    pub predicate_key: Option<String>,
    pub line: usize,
}

impl Default for SqlCreateIndexMetadata {
    fn default() -> Self {
        Self {
            table_name: String::new(),
            name: None,
            leading_column: None,
            columns: Vec::new(),
            include_columns: Vec::new(),
            access_method: "btree".to_string(),
            unique: false,
            has_predicate: false,
            not_null_predicate_column: None,
            predicate_key: None,
            line: 1,
        }
    }
}

/// One `ALTER TABLE … ADD COLUMN`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlAddColumnMetadata {
    pub table_name: String,
    /// Decoded last identifier component, preserving dots inside quoted names.
    pub unqualified_table_name: String,
    pub column_name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<String>,
    /// Whether the column is computed by a GENERATED expression.
    pub is_generated: bool,
    pub line: usize,
}

/// One foreign key from `CREATE TABLE` or `ALTER TABLE`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlForeignKeyMetadata {
    pub table_name: String,
    pub column_names: Vec<String>,
    pub referenced_table_name: String,
    pub delete_action: Option<String>,
    pub line: usize,
}

/// A named constraint add or `VALIDATE CONSTRAINT`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlNamedConstraint {
    pub table_name: String,
    pub name: String,
    pub line: usize,
}

/// An unnamed `ALTER TABLE ADD` FOREIGN KEY or CHECK.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlUnnamedConstraint {
    pub table_name: String,
    pub kind: String,
    pub line: usize,
}

/// One schema-level statement kind recorded from PostgreSQL SQL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlStatementKind {
    pub kind: String,
    pub line: usize,
}

/// A syntactically recoverable PostgreSQL parameter change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlSettingUse {
    pub name: String,
    pub line: usize,
}

/// Combined schema and embedded-SQL facts for one request.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PostgresFacts {
    pub schema: Vec<SqlSchemaFileFacts>,
    pub embedded: Vec<EmbeddedSqlFileFacts>,
    pub statements: Vec<crate::codebase::postgres::statement_facts::SqlStatementFileFacts>,
}

/// Path-aware extractor failure. Extractors return this instead of panicking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostgresFactError {
    pub path: Option<PathBuf>,
    pub message: String,
}

impl PostgresFactError {
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            path: None,
            message: message.into(),
        }
    }

    pub fn for_path(path: &Path, message: impl Into<String>) -> Self {
        Self {
            path: Some(path.to_path_buf()),
            message: message.into(),
        }
    }
}

impl fmt::Display for PostgresFactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.path {
            Some(path) => write!(f, "{}: {}", path.display(), self.message),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for PostgresFactError {}
