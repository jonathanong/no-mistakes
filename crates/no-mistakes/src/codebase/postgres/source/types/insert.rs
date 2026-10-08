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
    Columns {
        columns: Vec<PostgresSqlIdentifier>,
    },
    Constraint {
        name: PostgresSqlName,
    },
    Expressions {
        expressions: Vec<PostgresSqlExpression>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "operatorClasses")]
        operator_classes: Option<Vec<Option<PostgresSqlArbiterOperatorClass>>>,
    },
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<PostgresSqlAssignmentTarget>,
    pub expression: PostgresSqlExpression,
    pub provenance: PostgresSqlInsertProvenance,
    pub span: Option<PostgresSqlSpan>,
    pub complete: bool,
}

/// Syntactic index target; no catalog or replay-safety resolution is implied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlAssignmentTarget {
    pub base: PostgresSqlExpression,
    pub subscripts: Vec<PostgresSqlExpression>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indirection: Option<Vec<PostgresSqlAssignmentStep>>,
    pub span: Option<PostgresSqlSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlArbiterOperatorClass {
    pub name: PostgresSqlName,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<PostgresSqlArbiterParameter>,
    pub span: Option<PostgresSqlSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PostgresSqlAssignmentStep {
    Subscript {
        expression: PostgresSqlExpression,
        span: Option<PostgresSqlSpan>,
    },
    Field {
        name: PostgresSqlIdentifier,
        span: Option<PostgresSqlSpan>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PostgresSqlInsertProvenance {
    TargetColumn,
    ExcludedColumn,
    Literal,
    Placeholder,
    /// A function expression derives its result rather than naming one atomic source.
    Derived,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlArbiterParameter {
    pub name: PostgresSqlIdentifier,
    pub value: PostgresSqlExpression,
}
