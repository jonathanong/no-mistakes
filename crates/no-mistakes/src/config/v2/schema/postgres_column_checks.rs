use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct PostgresColumnCheckConstraintsOptions {
    pub schema_catalog_path: String,
    pub columns: Vec<PostgresColumnCheckRequirement>,
    pub allow: Vec<PostgresColumnCheckAllowEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresColumnCheckRequirement {
    pub pattern: String,
    pub types: Vec<String>,
    pub require: PostgresColumnCheckPredicate,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PostgresColumnCheckPredicate {
    NonNegative,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresColumnCheckAllowEntry {
    pub column: String,
    pub reason: String,
}
