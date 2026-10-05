use anyhow::{bail, Result};

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

    /// Resolve the executor selection written in one rule's options.
    ///
    /// A rule that scans executor calls must select them: with neither
    /// `importSpecifier` (blank counts as absent) nor `executorNames` it would
    /// silently scan nothing, so that is a configuration error. An explicit
    /// `executorNames: []` opts out for rules that should only scan SQL files
    /// and native SQL.
    pub fn for_rule(
        rule_id: &str,
        import_specifier: Option<&str>,
        executor_names: Option<&[String]>,
    ) -> Result<Self> {
        let specifier = import_specifier.unwrap_or_default();
        if specifier.is_empty() && executor_names.is_none() {
            bail!(
                "{rule_id} option importSpecifier: set importSpecifier (or executorNames) to \
select executor calls; set executorNames: [] to scan only SQL files and native SQL \
(see docs/migrations/explicit-postgres-executors.md)"
            );
        }
        Ok(Self::configured(
            specifier,
            executor_names.unwrap_or_default(),
        ))
    }
}

#[cfg(test)]
mod tests;
