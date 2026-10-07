use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlInsert {
    pub table: Option<PostgresSqlName>,
    pub alias: Option<PostgresSqlIdentifier>,
    pub columns: Vec<PostgresSqlName>,
    pub columns_omitted: bool,
    pub source: PostgresSqlInsertSource,
    pub on_conflict: Option<PostgresSqlConflict>,
    pub span: Option<PostgresSqlSpan>,
    pub complete: bool,
    pub diagnostics: Vec<PostgresSqlDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PostgresSqlInsertSource {
    Values {
        rows: Vec<Vec<PostgresSqlExpression>>,
        span: Option<PostgresSqlSpan>,
    },
    Select {
        query: PostgresSqlQuery,
        span: Option<PostgresSqlSpan>,
    },
    DefaultValues,
    Unsupported {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlConflict {
    pub target: PostgresSqlConflictTarget,
    pub predicate: Option<PostgresSqlExpression>,
    pub action: PostgresSqlConflictAction,
    pub span: Option<PostgresSqlSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PostgresSqlConflictTarget {
    Omitted,
    Columns { columns: Vec<PostgresSqlIdentifier> },
    Constraint { name: PostgresSqlName },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PostgresSqlConflictAction {
    DoNothing,
    DoUpdate {
        assignments: Vec<PostgresSqlInsertAssignment>,
        predicate: Option<Box<PostgresSqlExpression>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlInsertAssignment {
    pub columns: Vec<PostgresSqlName>,
    pub expression: PostgresSqlExpression,
    pub provenance: PostgresSqlInsertProvenance,
    pub span: Option<PostgresSqlSpan>,
    pub complete: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PostgresSqlInsertProvenance {
    TargetColumn,
    ExcludedColumn,
    Literal,
    Placeholder,
    Unresolved,
}
