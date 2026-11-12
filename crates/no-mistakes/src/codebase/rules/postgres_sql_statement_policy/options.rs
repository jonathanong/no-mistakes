use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    pub(crate) sql_include: Vec<String>,
    #[serde(default)]
    pub(crate) sql_include_locked: bool,
    pub(crate) banned_statements: Vec<String>,
    pub(crate) banned_settings: Vec<String>,
    pub(crate) import_specifier: String,
    pub(crate) executor_names: Vec<String>,
    pub(crate) extra_executor_names: Option<Vec<String>>,
    pub(crate) exclude_executor_names: Vec<String>,
    pub(crate) executor_factory_names: Vec<String>,
    pub(crate) executor_type_names: Vec<String>,
    pub(crate) trusted_sql_tags: Vec<crate::codebase::postgres::TrustedSqlTag>,
    pub(crate) unanalyzable_sql: String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            exclude: Vec::new(),
            sql_include: Vec::new(),
            sql_include_locked: false,
            banned_statements: super::DEFAULT_BANNED
                .iter()
                .map(|kind| (*kind).into())
                .collect(),
            banned_settings: Vec::new(),
            import_specifier: String::new(),
            executor_names: Vec::new(),
            extra_executor_names: None,
            exclude_executor_names: Vec::new(),
            executor_factory_names: Vec::new(),
            executor_type_names: Vec::new(),
            trusted_sql_tags: Vec::new(),
            unanalyzable_sql: String::new(),
        }
    }
}
