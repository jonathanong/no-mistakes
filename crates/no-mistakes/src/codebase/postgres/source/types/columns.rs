use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlType {
    pub sql: String,
    pub name: Option<PostgresSqlName>,
    pub builtin: Option<String>,
    pub modifiers: Vec<String>,
    pub array_dimensions: Vec<Option<u64>>,
    pub fields: Vec<PostgresSqlColumn>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlColumn {
    pub name: PostgresSqlIdentifier,
    pub data_type: PostgresSqlType,
    pub nullable: bool,
    pub default: Option<PostgresSqlExpression>,
    pub generated: Option<PostgresSqlGeneratedColumn>,
    pub identity: Option<PostgresSqlIdentityColumn>,
    pub constraints: Vec<PostgresSqlConstraint>,
    pub span: Option<PostgresSqlSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresSqlGeneratedColumn {
    pub expression: PostgresSqlExpression,
    pub storage: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresSqlIdentityColumn {
    pub mode: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresSqlConstraint {
    pub kind: PostgresSqlConstraintKind,
    pub name: Option<PostgresSqlIdentifier>,
    pub columns: Vec<PostgresSqlIdentifier>,
    pub expression: Option<PostgresSqlExpression>,
    pub referenced_table: Option<PostgresSqlName>,
    pub referenced_columns: Vec<PostgresSqlIdentifier>,
    pub on_delete: Option<String>,
    pub on_update: Option<String>,
    pub characteristics: Option<String>,
    pub sql: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PostgresSqlConstraintKind {
    PrimaryKey,
    Unique,
    ForeignKey,
    Check,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PostgresSqlAlterOperation {
    AddColumn {
        column: Box<PostgresSqlColumn>,
    },
    AlterColumnType {
        column: PostgresSqlIdentifier,
        #[serde(rename = "dataType")]
        data_type: PostgresSqlType,
        using: Option<PostgresSqlExpression>,
    },
    SetDefault {
        column: PostgresSqlIdentifier,
        expression: PostgresSqlExpression,
    },
    DropDefault {
        column: PostgresSqlIdentifier,
    },
    SetNotNull {
        column: PostgresSqlIdentifier,
    },
    DropNotNull {
        column: PostgresSqlIdentifier,
    },
    AddConstraint {
        constraint: Box<PostgresSqlConstraint>,
        #[serde(rename = "notValid")]
        not_valid: bool,
    },
    ValidateConstraint {
        name: PostgresSqlIdentifier,
    },
    Other {
        sql: String,
    },
}
