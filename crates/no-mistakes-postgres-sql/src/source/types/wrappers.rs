use super::*;

/// Execution classification belongs to the enclosing syntax, not consumer policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PostgresSqlExecution {
    NonExecuting,
    ExecutesForAnalysis,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PostgresSqlWrapperKind {
    Explain,
    Prepare,
    FunctionDeclaration,
}

/// Children are ordered source occurrences governed by the wrapper execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlWrapper {
    pub wrapper_kind: PostgresSqlWrapperKind,
    pub execution: PostgresSqlExecution,
    pub statements: Vec<PostgresSqlStatement>,
    pub span: Option<PostgresSqlSpan>,
    pub complete: bool,
    pub diagnostics: Vec<PostgresSqlDiagnostic>,
}
