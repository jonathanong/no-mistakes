#[derive(Clone, Default)]
struct ExportResolutionTrace {
    visited: Vec<(std::path::PathBuf, String)>,
    failures: std::collections::BTreeSet<std::path::PathBuf>,
}

impl std::ops::Deref for ExportResolutionTrace {
    type Target = Vec<(std::path::PathBuf, String)>;
    fn deref(&self) -> &Self::Target { &self.visited }
}

impl std::ops::DerefMut for ExportResolutionTrace {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.visited }
}

impl ExportResolutionTrace {
    fn record_source(&mut self, facts: &dyn TsFactLookup, path: &std::path::Path, values: bool) {
        if facts.get_ts_facts(path).is_some_and(|facts| facts.parse_error.is_some() || facts.operational_error.is_some()
            || values && facts.value_flow.incomplete_reason.is_some()) {
            self.failures.insert(path.to_path_buf());
        }
    }

    fn cache_failures(&self, indexes: &CallableResolutionIndexes, key: &(std::path::PathBuf, String)) {
        if !self.failures.is_empty() {
            indexes.export_prerequisites.insert(key.clone(), std::sync::Arc::new(self.failures.clone()));
        }
    }
}
