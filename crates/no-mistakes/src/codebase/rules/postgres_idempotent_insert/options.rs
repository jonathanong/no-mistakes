use serde::Deserialize;
use std::collections::HashMap;

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) sql_include: Vec<String>,
    #[serde(default)]
    pub(crate) sql_include_locked: bool,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
    pub(crate) executor_factory_names: Vec<String>,
    pub(crate) executor_type_names: Vec<String>,
    pub(crate) trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    pub(crate) unanalyzable_sql: String,
    #[serde(default = "default_true")]
    pub(crate) scan_embedded: bool,
    #[serde(default = "default_true")]
    pub(crate) check_convergence: bool,
    #[serde(default = "default_true")]
    pub(crate) check_volatility: bool,
    #[serde(default = "default_true")]
    pub(crate) check_arbiter: bool,
    #[serde(default = "default_true")]
    pub(crate) check_triggers: bool,
    #[serde(default = "default_true")]
    pub(crate) check_generated: bool,
    pub(crate) replay_safe_trigger_functions: Vec<String>,
    pub(crate) trigger_written_columns: HashMap<String, Vec<String>>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            exclude: Vec::new(),
            sql_include: Vec::new(),
            sql_include_locked: false,
            import_specifier: String::new(),
            executor_names: Vec::new(),
            executor_factory_names: Vec::new(),
            executor_type_names: Vec::new(),
            trusted_sql_tags: Vec::new(),
            unanalyzable_sql: String::new(),
            scan_embedded: true,
            check_convergence: true,
            check_volatility: true,
            check_arbiter: true,
            check_triggers: true,
            check_generated: true,
            replay_safe_trigger_functions: Vec::new(),
            trigger_written_columns: HashMap::new(),
        }
    }
}
