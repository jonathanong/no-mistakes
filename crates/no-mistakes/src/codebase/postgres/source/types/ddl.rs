use super::*;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlDrop {
    pub object_type: String,
    pub names: Vec<PostgresSqlName>,
    pub table: Option<PostgresSqlName>,
    pub signatures: Vec<String>,
    pub if_exists: bool,
    pub cascade: bool,
    pub restrict: bool,
    pub temporary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlView {
    pub name: PostgresSqlName,
    pub columns: Vec<PostgresSqlIdentifier>,
    pub materialized: bool,
    pub temporary: bool,
    pub or_replace: bool,
    pub query: String,
    pub dependencies: Vec<PostgresSqlName>,
    pub dependencies_complete: bool,
    pub functions: Vec<PostgresSqlFunctionReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlTrigger {
    pub name: PostgresSqlName,
    pub table: PostgresSqlName,
    pub timing: Option<String>,
    pub events: Vec<String>,
    pub for_each: Option<String>,
    pub condition: Option<PostgresSqlExpression>,
    pub function: Option<PostgresSqlName>,
    pub arguments: Vec<String>,
    pub constraint: bool,
    pub or_replace: bool,
    pub referenced_table: Option<PostgresSqlName>,
    pub transitions: Vec<PostgresSqlTriggerTransition>,
    pub execution_kind: Option<String>,
    pub characteristics: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlFunction {
    pub wrapper: PostgresSqlWrapper,
    pub name: PostgresSqlName,
    pub arguments: Vec<PostgresSqlFunctionArgument>,
    pub return_type: Option<PostgresSqlType>,
    pub returns_set: bool,
    pub language: Option<String>,
    pub behavior: Option<String>,
    pub body_sql: Option<String>,
    pub or_replace: bool,
    pub temporary: bool,
    pub called_on_null: Option<String>,
    pub parallel: Option<String>,
    pub configuration: Vec<String>,
    pub security: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlFunctionArgument {
    pub name: Option<PostgresSqlIdentifier>,
    pub mode: Option<String>,
    pub data_type: PostgresSqlType,
    pub default: Option<PostgresSqlExpression>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresSqlTriggerTransition {
    pub kind: String,
    pub name: PostgresSqlName,
}
