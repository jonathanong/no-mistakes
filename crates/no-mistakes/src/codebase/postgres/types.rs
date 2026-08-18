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
}

/// Schema facts for one SQL file read through [`crate::codebase::ts_source::SourceStore`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlSchemaFileFacts {
    pub path: PathBuf,
    pub tables: Vec<SqlCreateTableMetadata>,
}

/// Combined schema and embedded-SQL facts for one request.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PostgresFacts {
    pub schema: Vec<SqlSchemaFileFacts>,
    pub embedded: Vec<EmbeddedSqlFileFacts>,
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
