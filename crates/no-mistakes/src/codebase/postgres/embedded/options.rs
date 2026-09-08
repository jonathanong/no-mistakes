const DEFAULT_IMPORT_SPECIFIER: &str = "@data-stores/psql";
const DEFAULT_EXECUTOR_NAMES: &[&str] = &["query", "read", "write"];

/// Configurable executor import matching.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EmbeddedSqlOptions {
    pub import_specifier: String,
    pub executor_names: Vec<String>,
}

impl EmbeddedSqlOptions {
    /// Apply the public defaults and canonicalize executor names so the same
    /// configured projection has one request-scoped identity.
    pub fn configured(import_specifier: &str, executor_names: &[String]) -> Self {
        let defaults = Self::default();
        let mut options = Self {
            import_specifier: if import_specifier.is_empty() {
                defaults.import_specifier
            } else {
                import_specifier.to_string()
            },
            executor_names: if executor_names.is_empty() {
                defaults.executor_names
            } else {
                executor_names.to_vec()
            },
        };
        options.executor_names.sort();
        options.executor_names.dedup();
        options
    }
}

impl Default for EmbeddedSqlOptions {
    fn default() -> Self {
        Self {
            import_specifier: DEFAULT_IMPORT_SPECIFIER.to_string(),
            executor_names: DEFAULT_EXECUTOR_NAMES
                .iter()
                .map(|name| (*name).to_string())
                .collect(),
        }
    }
}
