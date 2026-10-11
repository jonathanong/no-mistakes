use super::SelectedRunners;
use crate::config::v2::NoMistakesConfig;

pub(crate) fn needs_vitest_configs(config: &NoMistakesConfig) -> bool {
    SelectedRunners::for_config(config).includes_vitest()
}

pub(crate) fn needs_playwright_configs(config: &NoMistakesConfig) -> bool {
    SelectedRunners::for_config(config).includes_all()
        || config
            .rule_applications(super::super::RULE_ID)
            .iter()
            .any(|rule| {
                rule.tests.playwright.iter().any(|name| {
                    config
                        .tests
                        .playwright
                        .projects
                        .get(name)
                        .is_none_or(|policy| policy.include.is_empty())
                })
            })
}
