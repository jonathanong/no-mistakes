use super::{
    PLAYWRIGHT_COVERAGE, PLAYWRIGHT_PREFER_TEST_ID_LOCATORS, PLAYWRIGHT_UNIQUE_HTML_IDS,
    PLAYWRIGHT_UNIQUE_TEST_IDS,
};
use crate::config::v2::NoMistakesConfig;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Default)]
pub(super) struct RuleSelection {
    pub(super) playwright_project: Option<String>,
    pub(super) coverage: bool,
    pub(super) unique_test_ids: bool,
    pub(super) unique_html_ids: bool,
    pub(super) prefer_test_id_locators: bool,
}

pub(super) fn rule_selections(config: &NoMistakesConfig) -> Vec<RuleSelection> {
    let mut by_project = BTreeMap::<Option<String>, RuleSelection>::new();
    add_rule_selections(
        config,
        PLAYWRIGHT_COVERAGE,
        |selection| selection.coverage = true,
        &mut by_project,
    );
    add_rule_selections(
        config,
        PLAYWRIGHT_UNIQUE_TEST_IDS,
        |selection| selection.unique_test_ids = true,
        &mut by_project,
    );
    add_rule_selections(
        config,
        PLAYWRIGHT_UNIQUE_HTML_IDS,
        |selection| selection.unique_html_ids = true,
        &mut by_project,
    );
    add_rule_selections(
        config,
        PLAYWRIGHT_PREFER_TEST_ID_LOCATORS,
        |selection| selection.prefer_test_id_locators = true,
        &mut by_project,
    );
    by_project.into_values().collect()
}

fn add_rule_selections(
    config: &NoMistakesConfig,
    rule_id: &str,
    apply: impl Fn(&mut RuleSelection) + Copy,
    by_project: &mut BTreeMap<Option<String>, RuleSelection>,
) {
    for rule in config.rule_applications(rule_id) {
        let projects: BTreeSet<Option<String>> = if rule.tests.playwright.is_empty() {
            [None].into_iter().collect()
        } else {
            rule.tests
                .playwright
                .iter()
                .map(|project| Some(project.clone()))
                .collect()
        };
        for project in projects {
            let selection = by_project
                .entry(project.clone())
                .or_insert_with(|| RuleSelection {
                    playwright_project: project,
                    ..RuleSelection::default()
                });
            apply(selection);
        }
    }
}
