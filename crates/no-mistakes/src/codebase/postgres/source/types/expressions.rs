use super::*;

/// Typed syntactic root. Wrappers remain explicit; nested references never imply a root call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PostgresSqlExpressionRoot {
    ColumnReference {
        name: PostgresSqlName,
    },
    FunctionCall {
        name: PostgresSqlName,
        arguments: Vec<PostgresSqlCallArgument>,
        #[serde(rename = "argumentsComplete")]
        arguments_complete: bool,
        /// Bare SQL value functions have no parentheses, unlike ordinary calls.
        syntax: PostgresSqlFunctionSyntax,
        /// DISTINCT, FILTER, OVER, and other call modifiers are retained as SQL.
        modifiers: Vec<String>,
    },
    Parenthesized {
        expression: Box<PostgresSqlExpressionRoot>,
    },
    Cast {
        #[serde(rename = "dataType")]
        data_type: String,
        expression: Box<PostgresSqlExpressionRoot>,
    },
    Literal {
        sql: String,
    },
    Binary {
        operator: String,
    },
    Unary {
        operator: String,
        expression: Box<PostgresSqlExpressionRoot>,
    },
    Case,
    Subquery,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PostgresSqlFunctionSyntax {
    Call,
    Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlCallArgument {
    pub name: Option<PostgresSqlIdentifier>,
    pub sql: String,
    pub span: Option<PostgresSqlSpan>,
    pub root: PostgresSqlExpressionRoot,
}
