use super::SqlWriteColumns;

/// A literal top-level JSON object key assigned by an executed INSERT or UPDATE.
/// Column ownership and same-row foreign keys are resolved from the prepared catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlJsonWriteFact {
    pub table: String,
    pub column: Option<String>,
    /// Unknown after an unresolved wildcard in an INSERT SELECT projection.
    pub column_index: Option<usize>,
    pub written_columns: SqlWriteColumns,
    pub key: String,
    pub line: usize,
    pub column_position: usize,
}
