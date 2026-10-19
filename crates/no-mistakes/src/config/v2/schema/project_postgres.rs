use serde::{Deserialize, Serialize};

/// PostgreSQL locations and executor targets declared once on a project.
///
/// `sqlInclude` and `schemaCatalogPath` are project-root-relative, like
/// [`super::Project::include`]. Rule options stay repository-relative; config
/// load prefixes `project.root` when it copies these into a rule.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ProjectPostgresConfig {
    pub sql_include: Vec<String>,
    pub schema_catalog_path: Option<String>,
    pub unanalyzable_sql: Option<String>,
    pub executors: Option<ProjectPostgresExecutors>,
}

/// Executor module, factories, types, and trusted SQL tags for one project.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ProjectPostgresExecutors {
    pub import_specifier: String,
    pub factory_names: Vec<String>,
    pub type_names: Vec<String>,
    pub trusted_sql_tags: Vec<ProjectPostgresTrustedSqlTag>,
}

/// A parameterized SQL tag imported from `module`.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectPostgresTrustedSqlTag {
    pub module: String,
    pub name: String,
}
