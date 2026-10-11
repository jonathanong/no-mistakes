use super::{build_globset, ConfigSetupData, TestFilter};
use crate::codebase::check_facts::RunnerConfigFacts;
use crate::config::v2::NoMistakesConfig;
use crate::integration_tests::types::VitestSetupField;
use anyhow::Result;
use std::path::Path;

pub(super) fn from_facts(
    config: &NoMistakesConfig,
    path: &Path,
    visible: &crate::fx::PathSet,
    facts: &RunnerConfigFacts,
) -> Result<Vec<ConfigSetupData>> {
    let file_facts = facts.get(path).ok_or_else(|| {
        anyhow::anyhow!(
            "missing prepared Vitest config projects for {}",
            path.display()
        )
    })?;
    let projects = file_facts
        .vitest_projects()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "missing prepared Vitest config projects for {}",
                path.display()
            )
        })?
        .map_err(anyhow::Error::msg)?;
    if projects.is_empty() && !file_facts.has_config_export() {
        // Named-only helpers supply no runner execution or global setup mocks.
        return Ok(Vec::new());
    }
    let rules = config.rule_applications(super::super::RULE_ID);
    let all = super::project_setups::all_runner_projects(config);
    let selected = rules
        .iter()
        .flat_map(|rule| &rule.tests.vitest)
        .collect::<std::collections::HashSet<_>>();
    projects
        .into_iter()
        .filter(|project| {
            all || project
                .policy_name
                .as_ref()
                .is_some_and(|name| selected.contains(name))
                || (project.policy_name.is_none() && !selected.is_empty())
        })
        .map(|project| {
            // Match test discovery: an explicit named policy owns its universe,
            // while the recovered runner still supplies that project's setups.
            let (include, exclude) = project
                .policy_name
                .as_ref()
                .and_then(|name| config.tests.vitest.projects.get(name))
                .filter(|policy| !policy.include.is_empty())
                .map(|policy| (&policy.include, &policy.exclude))
                .unwrap_or((&project.include, &project.exclude));
            Ok(ConfigSetupData {
                workspace: project.workspace,
                filter: TestFilter {
                    alternatives: Vec::new(),
                    include: build_globset(include)?,
                    include_regex: Vec::new(),
                    exclude: build_globset(exclude)?,
                    required: None,
                },
                // The runner parser has already applied extends, independent config
                // files, explicit empty overrides, and project-relative resolution.
                setup_files: project
                    .vitest_setup
                    .into_iter()
                    .filter(|setup| {
                        setup.field == VitestSetupField::SetupFiles
                            && !setup.config_extends_provenance
                    })
                    .filter_map(|setup| setup.resolved_path)
                    .filter(|path| visible.contains(path))
                    .collect(),
                explicit_project: false,
                runner: super::discovery::Runner::Vitest,
                project_name: project.policy_name,
            })
        })
        .collect::<Result<Vec<_>>>()
}
