use super::patterns::materialize_patterns;
use super::ParsedPlaywrightConfig;
use crate::integration_tests::types::ConfigProject;
use std::path::Path;

impl ParsedPlaywrightConfig {
    pub(in crate::integration_tests) fn into_projects_from_visible(
        self,
        root: &Path,
        config: &str,
        visible: Option<&crate::fx::PathSet>,
    ) -> anyhow::Result<Vec<ConfigProject>> {
        anyhow::ensure!(visible.is_some() || self.projects.iter().all(|project| project.test_match.regexes.is_empty() && project.test_ignore.regexes.is_empty()), "{config}: Playwright RegExp matchers require the caller-owned visible file inventory; pass the prepared inventory to project projection");
        let empty = crate::fx::PathSet::default();
        let visible = visible.unwrap_or(&empty);
        Ok(self
            .projects
            .into_iter()
            .map(|project| {
                let test_dir = project.test_dir(root);
                let matcher_warnings = project
                    .test_match
                    .warnings
                    .iter()
                    .chain(project.test_ignore.warnings.iter())
                    .map(|warning| warning.render(root))
                    .collect();
                ConfigProject {
                    config: Some(config.to_string()),
                    workspace: false,
                    policy_name: project.policy_name,
                    runner_project_arg: project.runner_project_arg,
                    scope: Some(crate::codebase::ts_source::relative_slash_path(
                        root, &test_dir,
                    )),
                    include: materialize_patterns(
                        root,
                        &test_dir,
                        &project.test_match,
                        visible,
                        true,
                    ),
                    exclude: materialize_patterns(
                        root,
                        &test_dir,
                        &project.test_ignore,
                        visible,
                        false,
                    ),
                    declared_deadlines: project.deadlines,
                    matcher_warnings,
                    vitest_setup: Vec::new(),
                }
            })
            .collect())
    }
}
