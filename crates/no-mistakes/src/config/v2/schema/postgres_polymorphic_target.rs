use serde::{Deserialize, Serialize};

fn default_type_suffix() -> String {
    "_type".to_string()
}

fn default_id_suffix() -> String {
    "_id".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct PostgresPolymorphicTargetOptions {
    pub schema_catalog_path: String,
    #[serde(default = "default_type_suffix")]
    pub type_suffix: String,
    #[serde(default = "default_id_suffix")]
    pub id_suffix: String,
    pub allow: Vec<PostgresPolymorphicTargetAllowEntry>,
}

impl Default for PostgresPolymorphicTargetOptions {
    fn default() -> Self {
        Self {
            schema_catalog_path: String::new(),
            type_suffix: default_type_suffix(),
            id_suffix: default_id_suffix(),
            allow: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresPolymorphicTargetAllowEntry {
    pub column: String,
    pub reason: String,
}
