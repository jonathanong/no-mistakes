use serde::{Deserialize, Serialize};

mod columns;
mod ddl;
mod indexes;
pub use columns::*;
pub use ddl::*;
pub use indexes::*;

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
    DoBlock {
        block: PostgresSqlProceduralBlock,
    },
    Conditional {
        branches: Vec<PostgresSqlConditionalBranch>,
    },
    Other,
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
    DollarQuoted,
    SingleQuoted,
}
