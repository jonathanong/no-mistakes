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
            unavailable: false,
            results,
            analyses: BTreeMap::new(),
        })
    }
    pub(crate) fn parse_path_for_facts_with_session(
        &self,
        session: &crate::codebase::analysis_session::AnalysisSession,
        path: &Path,
    ) -> Option<RunnerConfigFileFacts> {
        if !self.contains(path) {
            return None;
        }
        let unavailable = !path.exists();
        if unavailable && self.deadline_runners.is_empty() {
            return None;
        }
        let source = match match &self.sources {
            Some(sources) => sources
                .read_path(path)
                .map_err(|error| anyhow::anyhow!("reading {}: {}", path.display(), error)),
            None => super::super::cache::read_request_source_with_session(session, path),
        } {
            Ok(source) => source,
            Err(error) => {
                return self.parse_error(path, error.to_string()).map(|mut facts| {
                    facts.unavailable = unavailable;
                    facts
                })
            }
        };
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            return Some(self.parse_json(path, &source));
        }
        match session.with_program(path, &source, |program, source| {
            self.parse_program(path, program, source)
                .expect("runner config path was prepared")
        }) {
            Ok(facts) => Some(facts),
            Err(error) => self.parse_error(path, error.to_string()),
        }
    }
}
