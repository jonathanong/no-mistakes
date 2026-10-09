use super::*;

/// Meaning of an immediate expression edge; `index` distinguishes repeated roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PostgresSqlExpressionChildRole {
    Argument,
    BinaryLeft,
    BinaryRight,
    CaseOperand,
    CaseWhenCondition,
    CaseWhenResult,
    CaseElse,
    CastOperand,
    FilterPredicate,
    ParenthesizedExpression,
    UnaryOperand,
    NullOperand,
    DistinctLeft,
    DistinctRight,
    Other,
}

/// Shallow node kind; recursive operands are represented once by `children`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PostgresSqlExpressionChildRoot {
    ColumnReference {
        name: PostgresSqlName,
    },
    FunctionCall {
        name: PostgresSqlName,
        #[serde(rename = "argumentsComplete")]
        arguments_complete: bool,
        syntax: PostgresSqlFunctionSyntax,
        modifiers: Vec<String>,
    },
    Parenthesized,
    Cast {
        #[serde(rename = "castKind")]
        cast_kind: String,
        #[serde(rename = "dataType")]
        data_type: String,
        #[serde(rename = "dataTypeFacts")]
        data_type_facts: PostgresSqlType,
    },
    NullTest {
        negated: bool,
    },
    Distinctness {
        negated: bool,
    },
    Parameter {
        placeholder: String,
    },
    TypedLiteral {
        #[serde(rename = "dataType")]
        data_type: String,
        value: String,
        sql: String,
    },
    Literal {
        sql: String,
        value: PostgresSqlLiteralValue,
    },
    Binary {
        operator: String,
    },
    Unary {
        operator: String,
    },
    Case,
    Subquery,
    Other,
}

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
        #[serde(rename = "dataTypeFacts")]
        data_type_facts: PostgresSqlType,
        expression: Box<PostgresSqlExpressionRoot>,
    },
    NullTest {
        negated: bool,
    },
    Distinctness {
        negated: bool,
    },
    Parameter {
        placeholder: String,
    },
    TypedLiteral {
        #[serde(rename = "dataType")]
        data_type: String,
        value: String,
        sql: String,
    },
    Literal {
        sql: String,
        value: PostgresSqlLiteralValue,
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

/// Parser-native literal values; numbers retain their exact decimal spelling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PostgresSqlLiteralValue {
    Null,
    String { value: String },
    Number { value: String },
    Boolean { value: bool },
    Other { sql: String },
}
