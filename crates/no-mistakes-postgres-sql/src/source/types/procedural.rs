use super::*;

/// Literal provenance uses original source coordinates; children use decoded SQL coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlLiteralExecute {
    pub literal_span: PostgresSqlSpan,
    pub body_encoding: PostgresSqlExecuteEncoding,
    pub decoded_sql: String,
    /// Expressions remain syntax in enclosing-source coordinates, never evaluated values.
    pub using: Vec<PostgresSqlExpression>,
    pub statements: Vec<PostgresSqlStatement>,
    pub diagnostics: Vec<PostgresSqlDiagnostic>,
    pub complete: bool,
}

/// Nested statements are procedural source occurrences, not guaranteed execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlProceduralBlock {
    pub language: String,
    pub body_encoding: PostgresSqlBodyEncoding,
    pub body_span: PostgresSqlSpan,
    pub statements: Vec<PostgresSqlStatement>,
    /// Classified source occurrences. Visibility is not execution.
    pub occurrences: Vec<PostgresSqlProceduralOccurrence>,
    pub diagnostics: Vec<PostgresSqlDiagnostic>,
    pub complete: bool,
}

/// One statically visible procedural occurrence and the occurrences it contains.
///
/// Conditions, loop bounds, and SQL values are not evaluated. A nested `dml`
/// occurrence means the source text contains that DML, not that it runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlProceduralOccurrence {
    pub kind: PostgresSqlProceduralOccurrenceKind,
    pub span: PostgresSqlSpan,
    pub occurrences: Vec<PostgresSqlProceduralOccurrence>,
}

/// Closed procedural classification.
///
/// `utility` and `controlFlow` are non-DML forms such as `CREATE TYPE` and
/// `IF`/`RAISE`. `dml` is statically visible `INSERT`, `UPDATE`, `DELETE`, or
/// `MERGE`. `dynamicExecute` is `EXECUTE` of a non-literal command.
/// `unknown` fails closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PostgresSqlProceduralOccurrenceKind {
    Utility,
    ControlFlow,
    Dml,
    DynamicExecute,
    Unknown,
}

/// Branch statements are source occurrences, not guaranteed execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlConditionalBranch {
    pub condition: Option<PostgresSqlExpression>,
    pub span: PostgresSqlSpan,
    pub statements: Vec<PostgresSqlStatement>,
}

/// The source slice retains the enclosing literal's encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PostgresSqlBodyEncoding {
    EscapedString,
    DollarQuoted,
    SingleQuoted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PostgresSqlExecuteEncoding {
    EscapedString,
    DollarQuoted,
    SingleQuoted,
    Concatenated,
}
