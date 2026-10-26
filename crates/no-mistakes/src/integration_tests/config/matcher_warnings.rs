use super::{configured_project, exact_project};
use crate::config::v2::NoMistakesConfig;
use crate::integration_tests::runner_config::{
    ParsedRunnerConfigs, PreparedIntegrationRunnerConfigs,
};
use crate::integration_tests::types::Framework;
use anyhow::Result;
use std::path::Path;

pub(in crate::integration_tests) fn selected_matcher_warnings(
    root: &Path,
    config: &NoMistakesConfig,
    plan: &PreparedIntegrationRunnerConfigs,
    parsed: &ParsedRunnerConfigs,
) -> Result<Vec<String>> {
    let mut warnings = Vec::new();
    for (framework, policies) in [
        (Framework::Playwright, &config.tests.playwright.projects),
        (Framework::Vitest, &config.tests.vitest.projects),
    ] {
        if policies
            .values()
            .all(|policy| policy.integration_suites.is_empty())
        {
            continue;
        }
        let projects = parsed.projects_for(plan, framework)?;
        for (name, policy) in policies {
            if policy.integration_suites.is_empty()
                || configured_project(root, name, policy).is_some()
            {
                continue;
            }
            warnings.extend(
                exact_project(framework, name, &projects)?
                    .matcher_warnings
                    .iter()
                    .cloned(),
            );
        }
    }
    warnings.sort();
    warnings.dedup();
    Ok(warnings)
}
