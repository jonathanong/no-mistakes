use crate::config::v2::NoMistakesConfig;
use crate::integration_tests::types::ConfigProject;
use std::collections::BTreeSet;

/// Warnings belong to the same effective projects selected by forbidden-call
/// roots or integration suites. An explicit policy include replaces an uncertain matcher.
pub(super) fn selected(config: &NoMistakesConfig, projects: &[ConfigProject]) -> Vec<String> {
    let selectors = config
        .rule_applications(super::super::forbidden_calls::RULE_ID)
        .into_iter()
        .filter_map(|application| {
            application
                .options
                .get("roots")
                .and_then(serde_yaml::Value::as_sequence)
        })
        .flatten()
        .filter_map(|root| root.get("playwright"))
        .collect::<Vec<_>>();
    let all = selectors
        .iter()
        .any(|selector| selector.as_bool() == Some(true));
    let mut names = selectors
        .iter()
        .filter_map(|selector| selector.as_sequence())
        .flatten()
        .filter_map(serde_yaml::Value::as_str)
        .collect::<BTreeSet<_>>();
    names.extend(
        config
            .tests
            .playwright
            .projects
            .iter()
            .filter(|(_, policy)| {
                !policy.integration_suites.is_empty() && policy.include.is_empty()
            })
            .map(|(name, _)| name.as_str()),
    );
    let mut warnings = projects
        .iter()
        .filter(|project| {
            all || project
                .policy_name
                .as_deref()
                .is_some_and(|name| names.contains(name))
        })
        .flat_map(|project| project.matcher_warnings.iter().cloned())
        .collect::<Vec<_>>();
    warnings.sort();
    warnings.dedup();
    warnings
}
