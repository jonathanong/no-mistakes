use super::*;

impl PreparedIntegrationRunnerConfigs {
    pub(crate) fn missing_paths(
        &self,
        facts: &crate::codebase::check_facts::RunnerConfigFacts,
    ) -> Vec<PathBuf> {
        self.specs
            .iter()
            .filter(|spec| {
                facts
                    .get(&spec.path)
                    .is_none_or(|facts| !facts.covers_spec(spec.framework, &spec.raw))
            })
            .map(|spec| spec.path.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}
