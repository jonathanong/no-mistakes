use super::*;

/// Syntactic data-modifying query body, ordered by source position, not execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlQueryStatement {
    pub ordinal: usize,
    pub cte_id: Option<usize>,
    pub query_scope_id: usize,
    pub parent_scope_id: Option<usize>,
    pub sql: String,
    pub span: Option<PostgresSqlSpan>,
    pub returning: Vec<PostgresSqlReturningItem>,
    pub complete: bool,
    pub unsupported: Vec<PostgresSqlQueryUnsupported>,
    #[serde(flatten)]
    pub facts: PostgresSqlQueryStatementKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PostgresSqlQueryStatementKind {
    Insert { insert: Box<PostgresSqlCteInsert> },
    Update { update: PostgresSqlCteUpdate },
    Delete { delete: PostgresSqlCteDelete },
    Merge { merge: PostgresSqlCteMerge },
    Unsupported { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlCteInsert {
    pub table: Option<PostgresSqlName>,
    pub alias: Option<PostgresSqlIdentifier>,
    pub columns: Vec<PostgresSqlName>,
    pub columns_omitted: bool,
    pub source: PostgresSqlCteInsertSource,
    pub on_conflict: Option<PostgresSqlConflict>,
    pub diagnostics: Vec<PostgresSqlDiagnostic>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PostgresSqlCteInsertSource {
    Values {
        rows: Vec<Vec<PostgresSqlExpression>>,
        span: Option<PostgresSqlSpan>,
    },
    Select {
        query_scope_id: usize,
        span: Option<PostgresSqlSpan>,
    },
    DefaultValues,
    Unsupported {
        reason: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlCteUpdate {
    pub target_relation_ids: Vec<usize>,
    pub from_relation_ids: Vec<usize>,
    pub assignments: Vec<PostgresSqlDmlAssignment>,
    pub predicate: Option<PostgresSqlExpression>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlCteDelete {
    pub target_relation_ids: Vec<usize>,
    pub using_relation_ids: Vec<usize>,
    pub predicate: Option<PostgresSqlExpression>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlCteMerge {
    pub target_relation_ids: Vec<usize>,
    pub source_relation_ids: Vec<usize>,
    pub predicate: PostgresSqlExpression,
    pub clauses: Vec<PostgresSqlMergeClause>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlDmlAssignment {
    pub columns: Vec<PostgresSqlName>,
    pub expression: PostgresSqlExpression,
    pub span: Option<PostgresSqlSpan>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlMergeClause {
    pub match_kind: String,
    pub predicate: Option<PostgresSqlExpression>,
    pub span: Option<PostgresSqlSpan>,
    pub action: PostgresSqlMergeAction,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PostgresSqlMergeAction {
    Insert {
        columns: Vec<PostgresSqlName>,
        rows: Vec<Vec<PostgresSqlExpression>>,
    },
    Update {
        assignments: Vec<PostgresSqlDmlAssignment>,
    },
    Delete,
    DoNothing,
    Unsupported {
        reason: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PostgresSqlReturningItem {
    Expression {
        expression: Box<PostgresSqlExpression>,
        alias: Option<PostgresSqlIdentifier>,
    },
    Wildcard {
        qualifier: Option<PostgresSqlName>,
        span: Option<PostgresSqlSpan>,
    },
    Unsupported {
        reason: String,
        span: Option<PostgresSqlSpan>,
    },
}
