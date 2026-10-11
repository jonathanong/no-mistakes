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
    let projects = facts
        .get(path)
        .and_then(|facts| facts.vitest_projects())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "missing prepared Vitest config projects for {}",
                path.display()
            )
        })?
        .map_err(anyhow::Error::msg)?;
    let rules = config.rule_applications(super::super::RULE_ID);
    let all = rules.is_empty()
        || rules.iter().any(|rule| {
            rule.applies_to_repository()
                || rule
                    .projects
                    .iter()
                    .any(|name| config.projects.contains_key(name))
        });
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
            // A recovered config cannot widen an explicitly declared policy's
            // setup scope, including its excluded test files.
            let policy_filter = project
                .policy_name
                .as_ref()
                .and_then(|name| config.tests.vitest.projects.get(name))
                .filter(|policy| !policy.include.is_empty())
                .map(|policy| {
                    Ok::<_, anyhow::Error>(TestFilter {
                        include: build_globset(&policy.include)?,
                        include_regex: Vec::new(),
                        exclude: build_globset(&policy.exclude)?,
                    })
                })
                .transpose()?;
            Ok(ConfigSetupData {
                policy_filter,
                filter: TestFilter {
                    include: build_globset(&project.include)?,
                    include_regex: Vec::new(),
                    exclude: build_globset(&project.exclude)?,
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
        .collect()
}
