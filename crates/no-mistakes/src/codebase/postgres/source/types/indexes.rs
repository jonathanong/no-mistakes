use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlIndex {
    pub name: Option<PostgresSqlName>,
    pub table: PostgresSqlName,
    pub method: String,
    pub unique: bool,
    pub nulls_distinct: bool,
    pub keys: Vec<PostgresSqlIndexKey>,
    pub include: Vec<PostgresSqlIdentifier>,
    pub predicate: Option<PostgresSqlExpression>,
    pub options: Vec<String>,
    pub structural_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlIndexKey {
    pub expression: PostgresSqlExpression,
    pub ascending: bool,
    pub nulls_first: bool,
    pub operator_class: Option<PostgresSqlName>,
}
