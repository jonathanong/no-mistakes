use serde::{Deserialize, Serialize};

mod columns;
mod ddl;
mod expressions;
mod indexes;
mod insert;
mod metadata;
mod query;
mod query_statements;
mod wrappers;
pub use columns::*;
pub use ddl::*;
pub use expressions::*;
pub use indexes::*;
pub use insert::*;
pub use metadata::*;
pub use query::*;
pub use query_statements::*;
pub use wrappers::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlSource {
    pub sql: String,
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlFacts {
    pub schema_version: u32,
    pub file_name: Option<String>,
    pub statements: Vec<PostgresSqlStatement>,
    pub diagnostics: Vec<PostgresSqlDiagnostic>,
}

/// Offsets count UTF-8 bytes; line/column count Unicode scalars from one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlPosition {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

/// The end position is exclusive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresSqlSpan {
    pub start: PostgresSqlPosition,
    pub end: PostgresSqlPosition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresSqlDiagnostic {
    pub message: String,
    pub span: Option<PostgresSqlSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlIdentifier {
    pub value: String,
    pub quoted: bool,
    pub identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresSqlName {
    pub parts: Vec<PostgresSqlIdentifier>,
    pub sql: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlExpression {
    pub sql: String,
    pub identity: String,
    pub span: Option<PostgresSqlSpan>,
    pub columns: Vec<PostgresSqlName>,
    pub functions: Vec<PostgresSqlFunctionReference>,
    pub root: PostgresSqlExpressionRoot,
    /// Ordered immediate child expressions; this is a recursive syntactic projection.
    pub children: Vec<PostgresSqlExpressionChild>,
    /// False when any expression child could not be projected without loss.
    pub children_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlExpressionChild {
    pub role: PostgresSqlExpressionChildRole,
    pub index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argument_name: Option<PostgresSqlIdentifier>,
    pub sql: String,
    pub span: Option<PostgresSqlSpan>,
    pub root: PostgresSqlExpressionChildRoot,
    pub children: Vec<PostgresSqlExpressionChild>,
    pub children_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresSqlFunctionReference {
    pub name: PostgresSqlName,
    pub span: Option<PostgresSqlSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresSqlStatement {
    pub ordinal: usize,
    pub span: PostgresSqlSpan,
    pub sql: String,
    #[serde(flatten)]
    pub facts: PostgresSqlStatementKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PostgresSqlStatementKind {
    Comment {
        comment: PostgresSqlComment,
    },
    AlterIndex {
        index: PostgresSqlAlterIndex,
    },
    Wrapper {
        wrapper: PostgresSqlWrapper,
    },
    Insert {
        insert: Box<PostgresSqlInsert>,
    },
    Select {
        query: PostgresSqlQuery,
    },
    CreateTable {
        table: PostgresSqlName,
        columns: Vec<PostgresSqlColumn>,
        constraints: Vec<PostgresSqlConstraint>,
        temporary: bool,
    },
    AlterTable {
        table: PostgresSqlName,
        operations: Vec<PostgresSqlAlterOperation>,
    },
    CreateIndex {
        index: PostgresSqlIndex,
    },
    CreateView {
        view: PostgresSqlView,
    },
    CreateTrigger {
        trigger: PostgresSqlTrigger,
    },
    CreateFunction {
        function: PostgresSqlFunction,
    },
    Drop {
        drop: PostgresSqlDrop,
    },
    LiteralExecute {
        execute: PostgresSqlLiteralExecute,
    },
    DoBlock {
        block: PostgresSqlProceduralBlock,
    },
    Conditional {
        branches: Vec<PostgresSqlConditionalBranch>,
    },
    Other,
}

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
    pub diagnostics: Vec<PostgresSqlDiagnostic>,
    pub complete: bool,
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
