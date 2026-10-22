use super::EmbeddedSqlKind;
use serde::{Deserialize, Serialize};

/// Supplied TypeScript or JavaScript source; no repository read is performed.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutedPostgresSource {
    pub file_name: String,
    pub source: String,
}

/// An explicit import edge between two supplied source files.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutedPostgresResolvedImport {
    pub from_file_name: String,
    pub specifier: String,
    pub to_file_name: String,
}

/// One executor call or one complete alternative of a conditional executor call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutedPostgresSql {
    pub file_name: String,
    pub line: u32,
    pub annotation: Option<String>,
    pub sql_text: Option<String>,
    /// `dynamic` may expose only a verified prefix. An incomplete statement
    /// needs `only_value_placeholders` before a consumer may parse its shape.
    pub complete: bool,
    /// SQL is structurally known except for generated value binds. Consumers
    /// must still parse the statement and reject writes or unsupported syntax.
    pub only_value_placeholders: bool,
    pub kind: EmbeddedSqlKind,
    /// Why the evaluator could not recover the whole statement. Generated
    /// value binds have their own provenance and do not use a failure reason.
    pub incomplete_reason: Option<String>,
}
