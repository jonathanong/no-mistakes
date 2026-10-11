use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlQuery {
    pub scopes: Vec<PostgresSqlQueryScope>,
    pub relations: Vec<PostgresSqlQueryRelation>,
    pub joins: Vec<PostgresSqlQueryJoin>,
    pub columns: Vec<PostgresSqlQueryColumn>,
    pub equalities: Vec<PostgresSqlQueryEquality>,
    pub exists: Vec<PostgresSqlQueryExists>,
    pub ctes: Vec<PostgresSqlQueryCte>,
    pub nested_statements: Vec<PostgresSqlQueryStatement>,
    pub unsupported: Vec<PostgresSqlQueryUnsupported>,
    pub complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlQueryScope {
    pub id: usize,
    pub parent_scope_id: Option<usize>,
    pub clause: PostgresSqlQueryClause,
    pub cte_definition_id: Option<usize>,
    pub set_operation: Option<String>,
    pub set_quantifier: Option<String>,
    pub span: Option<PostgresSqlSpan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PostgresSqlQueryClause {
    Root,
    SetBranch,
    Cte,
    From,
    Projection,
    Where,
    JoinOn,
    Having,
    GroupBy,
    OrderBy,
    Limit,
    Offset,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlQueryRelation {
    pub id: usize,
    pub scope_id: usize,
    pub kind: PostgresSqlQueryRelationKind,
    pub name: Option<PostgresSqlName>,
    pub alias: Option<PostgresSqlIdentifier>,
    pub column_aliases: Vec<PostgresSqlIdentifier>,
    pub cte_id: Option<usize>,
    pub subquery_scope_id: Option<usize>,
    pub members: Vec<usize>,
    pub lateral: bool,
    pub span: Option<PostgresSqlSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PostgresSqlQueryRelationKind {
    Table,
    Cte,
    Derived,
    Joined,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlQueryJoin {
    pub id: usize,
    pub scope_id: usize,
    pub kind: PostgresSqlQueryJoinKind,
    pub left: Vec<usize>,
    pub right: Vec<usize>,
    pub constraint: String,
    pub using_columns: Vec<PostgresSqlName>,
    pub span: Option<PostgresSqlSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PostgresSqlQueryJoinKind {
    Inner,
    Left,
    Right,
    Full,
    Cross,
    Semi,
    Anti,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlQueryColumn {
    pub scope_id: usize,
    pub clause: PostgresSqlQueryClause,
    pub name: PostgresSqlName,
    pub relation_id: Option<usize>,
    pub relation_scope_id: Option<usize>,
    pub resolution: PostgresSqlQueryColumnResolution,
    pub span: Option<PostgresSqlSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PostgresSqlQueryColumnResolution {
    Resolved,
    Unqualified,
    Unknown,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlPredicateContext {
    pub mandatory: bool,
    /// Whether this occurrence is a required conjunct after NOT inverts
    /// enclosing AND/OR operators. None means an opaque wrapper prevents proof.
    pub effective_mandatory: Option<bool>,
    pub under_or: bool,
    pub under_not: bool,
    pub under_case: bool,
    pub under_boolean_test: bool,
    pub under_other: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlQueryEquality {
    pub scope_id: usize,
    pub clause: PostgresSqlQueryClause,
    pub join_id: Option<usize>,
    pub left: Option<PostgresSqlQueryColumn>,
    pub right: Option<PostgresSqlQueryColumn>,
    pub context: PostgresSqlPredicateContext,
    pub span: Option<PostgresSqlSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlQueryExists {
    pub scope_id: usize,
    pub subquery_scope_id: usize,
    /// The EXISTS node's own flag. Wrapping `NOT` is not folded into this value.
    pub negated: bool,
    /// NOT operators that apply to this EXISTS: each wrapping `NOT`, through
    /// parentheses, plus one when `negated` is true.
    pub not_depth: u32,
    /// `Some(not_depth % 2 == 1)` unless `context.under_boolean_test`,
    /// `under_case`, or `under_other` is set. Those wrappers leave this `None`
    /// (JSON `null`, always present). Distinct from `negated` and `under_not`.
    pub effective_negated: Option<bool>,
    pub context: PostgresSqlPredicateContext,
    pub correlated: bool,
    pub correlations: Vec<PostgresSqlQueryColumn>,
    pub span: Option<PostgresSqlSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlQueryCte {
    pub id: usize,
    pub owner_scope_id: usize,
    pub query_scope_id: usize,
    pub name: PostgresSqlIdentifier,
    pub column_aliases: Vec<PostgresSqlIdentifier>,
    pub recursive: bool,
    pub referenced: bool,
    pub used: bool,
    pub cyclic: bool,
    pub span: Option<PostgresSqlSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlQueryUnsupported {
    pub scope_id: usize,
    pub clause: PostgresSqlQueryClause,
    pub reason: String,
    pub span: Option<PostgresSqlSpan>,
}
