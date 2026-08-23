use super::*;

impl PreparedIntegrationRunnerConfigs {
    pub(crate) fn parse_error(
        &self,
        path: &Path,
        message: String,
    ) -> Option<RunnerConfigFileFacts> {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        let results = self
            .specs
            .iter()
            .filter(|spec| spec.path == path)
            .map(|spec| ProjectResult {
                framework: spec.framework,
                raw: spec.raw.clone(),
                projects: Err(message.clone()),
            })
            .collect::<Vec<_>>();
        (!results.is_empty()).then_some(RunnerConfigFileFacts {
            results,
            analyses: BTreeMap::new(),
        })
    }
}
