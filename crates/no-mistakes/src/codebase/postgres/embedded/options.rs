use serde::Deserialize;

const DEFAULT_EXECUTOR_NAMES: &[&str] = &["query", "read", "write"];

/// A named import the user asserts is a parameterized SQL tag.
///
/// Matching is syntactic: `name` imported from `module` or a subpath of it.
/// Empty `module` or `name` matches nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustedSqlTag {
    pub module: String,
    pub name: String,
}

/// Configurable executor import matching. Empty defaults select no executors.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EmbeddedSqlOptions {
    pub import_specifier: String,
    pub executor_names: Vec<String>,
    /// Named imports whose call result bound to a local is an executor.
    pub executor_factory_names: Vec<String>,
    /// Imported type names whose annotated parameters are executors.
    pub executor_type_names: Vec<String>,
    /// Named imports trusted as parameterized SQL tags. Empty trusts none.
    pub trusted_sql_tags: Vec<TrustedSqlTag>,
}

impl EmbeddedSqlOptions {
    pub(crate) fn selects_executors(&self) -> bool {
        !self.executor_names.is_empty()
            || !self.executor_factory_names.is_empty()
            || !self.executor_type_names.is_empty()
    }

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
            executor_factory_names: Vec::new(),
            executor_type_names: Vec::new(),
            trusted_sql_tags: Vec::new(),
        }
    }

    /// Add scoped executor sources: factory imports and executor type imports.
    pub fn with_scoped_executors(mut self, factories: &[String], types: &[String]) -> Self {
        self.executor_factory_names = sorted_unique(factories);
        self.executor_type_names = sorted_unique(types);
        self
    }

    /// Opt in to parameterized tags. Order does not affect the profile key.
    pub fn with_trusted_sql_tags(mut self, tags: &[TrustedSqlTag]) -> Self {
        let mut tags = tags.to_vec();
        tags.sort();
        tags.dedup();
        self.trusted_sql_tags = tags;
        self
    }
}

fn sorted_unique(names: &[String]) -> Vec<String> {
    let mut names = names.to_vec();
    names.sort();
    names.dedup();
    names
}
