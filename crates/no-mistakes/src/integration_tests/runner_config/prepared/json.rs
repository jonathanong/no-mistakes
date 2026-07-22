use super::*;

impl PreparedIntegrationRunnerConfigs {
    pub(super) fn parse_json(&self, path: &Path, source: &str) -> Option<RunnerConfigFileFacts> {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        let specs = self
            .specs
            .iter()
            .filter(|spec| spec.path == path)
            .collect::<Vec<_>>();
        if specs.is_empty() {
            return None;
        }
        let config_dir = path.parent().unwrap_or(&self.root);
        let resolver = crate::codebase::ts_resolver::ScopedImportResolver::from_visible(
            &self.tsconfig_catalog,
            &self.visible_files,
        );
        let results = specs
            .into_iter()
            .map(|spec| ProjectResult {
                framework: spec.framework,
                raw: spec.raw.clone(),
                projects: if spec.framework == Framework::Vitest
                    && crate::integration_tests::is_vitest_project_array_path(&path)
                {
                    project_config::load_vitest_json_projects(ConfigProjectInput {
                        root: &self.root,
                        framework: spec.framework,
                        raw: &spec.raw,
                        path: &path,
                        source,
                        config_dir,
                        resolver: &resolver,
                    })
                    .map_err(|error| error.to_string())
                } else {
                    Err(format!(
                        "unsupported {} JSON config filename: {}; use vitest.workspace.json or vitest.projects.json",
                        spec.framework.as_str(),
                        path.display()
                    ))
                },
            })
            .collect();
        Some(RunnerConfigFileFacts {
            results,
            analyses: BTreeMap::new(),
        })
    }
}
