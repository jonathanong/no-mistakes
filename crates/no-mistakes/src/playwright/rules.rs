use crate::codebase::check_facts::{CheckFactMap, PlaywrightFactPlan};
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use crate::playwright::analysis::discover::discover_test_files;
use crate::playwright::analysis::pipeline::{
    analyze_selectors_with_policy, analyze_selectors_with_policy_and_facts, analyze_with_policy,
    analyze_with_policy_and_facts,
};
use crate::playwright::analysis::types::UniqueSelectorPolicy;
use crate::playwright::config;
use crate::playwright::playwright_config;
use crate::playwright::playwright_tests;
use crate::playwright::rule_findings::findings_from_report;
use anyhow::Result;
use filter::filter_rule_findings;
use selection::rule_selections;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

mod filter;
mod selection;

pub const PLAYWRIGHT_COVERAGE: &str = "playwright-coverage";
pub const PLAYWRIGHT_UNIQUE_TEST_IDS: &str = "playwright-unique-test-ids";
pub const PLAYWRIGHT_UNIQUE_HTML_IDS: &str = "playwright-unique-html-ids";
pub const PLAYWRIGHT_PREFER_TEST_ID_LOCATORS: &str = "playwright-prefer-test-id-locators";

pub fn configured(config: &NoMistakesConfig) -> bool {
    coverage_enabled(config)
        || unique_test_ids_enabled(config)
        || unique_html_ids_enabled(config)
        || prefer_test_id_locators_enabled(config)
}

pub fn check(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
) -> Result<Vec<RuleFinding>> {
    let selections = rule_selections(config);
    if selections.is_empty() {
        return Ok(Vec::new());
    }

    let mut findings = Vec::new();
    for selection in selections {
        let settings =
            config::load_settings(root, config_path, &[], selection.playwright_project.clone())?;
        let test_policy = playwright_tests::TestPolicy {
            assert_conditional_tests: false,
            allow_skipped_tests: false,
        };
        let unique_policy = unique_policy(selection.unique_test_ids, selection.unique_html_ids);
        let analysis = if selection.coverage {
            analyze_with_policy(root, &settings, test_policy, unique_policy)
        } else {
            analyze_selectors_with_policy(root, &settings, test_policy, unique_policy)
        }?;
        let report_findings = findings_from_report(
            &analysis,
            selection.coverage,
            selection.unique_test_ids,
            selection.unique_html_ids,
            selection.prefer_test_id_locators,
        );
        findings.extend(filter_rule_findings(root, config, report_findings)?);
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

pub fn fact_plan(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
) -> Result<Option<PlaywrightFactPlan>> {
    let selections = rule_selections(config);
    if selections.is_empty() {
        return Ok(None);
    }
    let mut navigation_helpers = Vec::new();
    let mut selector_regexes = None;
    let mut test_id_attributes_by_path = HashMap::new();
    for selection in selections {
        let settings = config::load_settings(root, config_path, &[], selection.playwright_project)?;
        if selector_regexes.is_none() {
            selector_regexes = Some(
                crate::playwright::selectors::compile_selector_regexes_with_html_ids(
                    &settings.selector_attributes,
                    &settings.component_selector_attributes,
                    settings.html_ids,
                ),
            );
            navigation_helpers = settings.navigation_helpers.clone();
        }
        let playwright = playwright_config::load_many(
            root,
            &settings.playwright_configs,
            settings.project.as_deref(),
        )?;
        for test_file in discover_test_files(root, &settings, &playwright)? {
            let attributes = test_file.test_id_attributes();
            let entry = test_id_attributes_by_path
                .entry(test_file.path)
                .or_insert_with(Vec::new);
            entry.extend(attributes);
            entry.sort();
            entry.dedup();
        }
    }
    Ok(Some(PlaywrightFactPlan {
        navigation_helpers,
        selector_regexes: Arc::new(
            selector_regexes.expect("configured Playwright rule has at least one selection"),
        ),
        test_id_attributes_by_path: Arc::new(test_id_attributes_by_path),
    }))
}

pub(crate) fn check_with_facts(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let selections = rule_selections(config);
    if selections.is_empty() {
        return Ok(Vec::new());
    }

    let mut findings = Vec::new();
    for selection in selections {
        let settings =
            config::load_settings(root, config_path, &[], selection.playwright_project.clone())?;
        let test_policy = playwright_tests::TestPolicy {
            assert_conditional_tests: false,
            allow_skipped_tests: false,
        };
        let unique_policy = unique_policy(selection.unique_test_ids, selection.unique_html_ids);
        let analysis = if selection.coverage {
            analyze_with_policy_and_facts(root, &settings, test_policy, unique_policy, facts)
        } else {
            analyze_selectors_with_policy_and_facts(
                root,
                &settings,
                test_policy,
                unique_policy,
                facts,
            )
        }?;
        let report_findings = findings_from_report(
            &analysis,
            selection.coverage,
            selection.unique_test_ids,
            selection.unique_html_ids,
            selection.prefer_test_id_locators,
        );
        findings.extend(filter_rule_findings(root, config, report_findings)?);
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

fn unique_policy(unique_test_ids: bool, unique_html_ids: bool) -> UniqueSelectorPolicy {
    UniqueSelectorPolicy {
        test_ids: unique_test_ids,
        html_ids: unique_html_ids,
        aggregate: false,
        configured_html_id_selector: false,
    }
}

fn coverage_enabled(config: &NoMistakesConfig) -> bool {
    config.rule_configured(PLAYWRIGHT_COVERAGE)
}

fn unique_test_ids_enabled(config: &NoMistakesConfig) -> bool {
    config.rule_configured(PLAYWRIGHT_UNIQUE_TEST_IDS)
}

fn unique_html_ids_enabled(config: &NoMistakesConfig) -> bool {
    config.rule_configured(PLAYWRIGHT_UNIQUE_HTML_IDS)
}

fn prefer_test_id_locators_enabled(config: &NoMistakesConfig) -> bool {
    config.rule_configured(PLAYWRIGHT_PREFER_TEST_ID_LOCATORS)
}

#[cfg(test)]
mod tests;
