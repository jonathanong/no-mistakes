use super::{EmbeddedSqlOptions, DEFAULT_EXECUTOR_NAMES};

impl EmbeddedSqlOptions {
    pub(crate) fn is_callable_export(&self, name: &str) -> bool {
        DEFAULT_EXECUTOR_NAMES.contains(&name)
            || self
                .executor_names
                .iter()
                .any(|candidate| candidate == name)
            || self
                .executor_factory_names
                .iter()
                .any(|candidate| candidate == name)
    }

    pub(crate) fn uses_legacy_executor_defaults(&self) -> bool {
        self.executor_names.is_empty() && self.exclude_executor_names.is_empty()
    }

    pub(crate) fn selects_executors(&self) -> bool {
        !self.executor_names.is_empty()
            || !self.executor_factory_names.is_empty()
            || !self.executor_type_names.is_empty()
    }
}
