const DEFAULT_EXECUTOR_NAMES: &[&str] = &["query", "read", "write"];

/// Configurable executor import matching. Empty defaults select no executors.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EmbeddedSqlOptions {
    pub import_specifier: String,
    pub executor_names: Vec<String>,
}

impl EmbeddedSqlOptions {
    /// A configured module enables standard executor names. Without a module,
    /// names must be explicit and may match named imports from any module.
    pub fn configured(import_specifier: &str, executor_names: &[String]) -> Self {
        let mut names = if executor_names.is_empty() && !import_specifier.is_empty() {
            DEFAULT_EXECUTOR_NAMES
                .iter()
                .map(|name| (*name).to_string())
                .collect()
        } else {
            executor_names.to_vec()
        };
        names.sort();
        names.dedup();
        Self {
            import_specifier: import_specifier.to_string(),
            executor_names: names,
        }
    }
}
