use crate::codebase::check_facts::{CheckFactMap, PlaywrightFactPlan};
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use crate::playwright::analysis::discover::discover_test_files;
use crate::playwright::analysis::pipeline::{analyze_with_policy, analyze_with_policy_and_facts};
use crate::playwright::analysis::types::{CoverageReport, DuplicateSelector, UniqueSelectorPolicy};
use crate::playwright::config;
use crate::playwright::playwright_config;
use crate::playwright::playwright_tests;
use crate::playwright::selectors::HTML_ID_ATTRIBUTE;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

pub const PLAYWRIGHT_COVERAGE: &str = "playwright-coverage";
pub const PLAYWRIGHT_UNIQUE_TEST_IDS: &str = "playwright-unique-test-ids";
pub const PLAYWRIGHT_UNIQUE_HTML_IDS: &str = "playwright-unique-html-ids";

pub fn configured(config: &NoMistakesConfig) -> bool {
    coverage_enabled(config) || unique_test_ids_enabled(config) || unique_html_ids_enabled(config)
}

pub fn check(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
) -> Result<Vec<RuleFinding>> {
    let coverage = coverage_enabled(config);
    let unique_test_ids = unique_test_ids_enabled(config);
    let unique_html_ids = unique_html_ids_enabled(config);
    if !coverage && !unique_test_ids && !unique_html_ids {
        return Ok(Vec::new());
    }

    let settings = config::load_settings(root, config_path, &[], None)?;
    let analysis = analyze_with_policy(
        root,
        &settings,
        playwright_tests::TestPolicy {
            assert_conditional_tests: false,
            allow_skipped_tests: false,
        },
        UniqueSelectorPolicy {
            test_ids: unique_test_ids,
            html_ids: unique_html_ids,
            aggregate: false,
            configured_html_id_selector: false,
        },
    )?;

    Ok(findings_from_report(
        &analysis.coverage,
        coverage,
        unique_test_ids,
        unique_html_ids,
    ))
}

pub fn fact_plan(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
) -> Result<Option<PlaywrightFactPlan>> {
    if !configured(config) {
        return Ok(None);
    }
    let settings = config::load_settings(root, config_path, &[], None)?;
    let playwright = playwright_config::load_many(
        root,
        &settings.playwright_configs,
        settings.project.as_deref(),
    )?;
    let test_files = discover_test_files(root, &settings, &playwright)?;
    let selector_regexes = crate::playwright::selectors::compile_selector_regexes_with_html_ids(
        &settings.selector_attributes,
        &settings.component_selector_attributes,
        settings.html_ids,
    );
    let test_id_attributes_by_path = test_files
        .into_iter()
        .map(|test_file| {
            let attributes = test_file.test_id_attributes();
            (test_file.path, attributes)
        })
        .collect::<HashMap<_, _>>();
    Ok(Some(PlaywrightFactPlan {
        navigation_helpers: settings.navigation_helpers,
        selector_regexes: Arc::new(selector_regexes),
        test_id_attributes_by_path: Arc::new(test_id_attributes_by_path),
    }))
}

pub(crate) fn check_with_facts(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    facts: &CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let coverage = coverage_enabled(config);
    let unique_test_ids = unique_test_ids_enabled(config);
    let unique_html_ids = unique_html_ids_enabled(config);
    if !coverage && !unique_test_ids && !unique_html_ids {
        return Ok(Vec::new());
    }

    let settings = config::load_settings(root, config_path, &[], None)?;
    let analysis = analyze_with_policy_and_facts(
        root,
        &settings,
        playwright_tests::TestPolicy {
            assert_conditional_tests: false,
            allow_skipped_tests: false,
        },
        UniqueSelectorPolicy {
            test_ids: unique_test_ids,
            html_ids: unique_html_ids,
            aggregate: false,
            configured_html_id_selector: false,
        },
        facts,
    )?;

    Ok(findings_from_report(
        &analysis.coverage,
        coverage,
        unique_test_ids,
        unique_html_ids,
    ))
}

fn coverage_enabled(config: &NoMistakesConfig) -> bool {
    config.rule_configured(PLAYWRIGHT_COVERAGE)
}

fn findings_from_report(
    report: &CoverageReport,
    coverage: bool,
    unique_test_ids: bool,
    unique_html_ids: bool,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    if coverage {
        findings.extend(coverage_findings(report));
    }
    if unique_test_ids || unique_html_ids {
        findings.extend(unique_findings(
            &report.duplicate_selectors,
            unique_test_ids,
            unique_html_ids,
        ));
    }
    findings.sort();
    findings.dedup();
    findings
}

fn unique_test_ids_enabled(config: &NoMistakesConfig) -> bool {
    config.rule_configured(PLAYWRIGHT_UNIQUE_TEST_IDS)
}

fn unique_html_ids_enabled(config: &NoMistakesConfig) -> bool {
    config.rule_configured(PLAYWRIGHT_UNIQUE_HTML_IDS)
}

fn coverage_findings(report: &CoverageReport) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for route in report.routes.iter().filter(|route| !route.covered) {
        findings.push(RuleFinding {
            rule: PLAYWRIGHT_COVERAGE.to_string(),
            file: route.file.clone(),
            line: 1,
            message: format!(
                "Next.js route `{}` is not covered by a Playwright navigation assertion",
                route.route
            ),
            import: None,
            target: Some(route.route.clone()),
        });
    }
    for selector in report.selectors.iter().filter(|selector| !selector.covered) {
        findings.push(RuleFinding {
            rule: PLAYWRIGHT_COVERAGE.to_string(),
            file: selector.file.clone(),
            line: 1,
            message: format!(
                "selector [{}=\"{}\"] is not covered by a Playwright locator",
                selector.attribute, selector.value
            ),
            import: None,
            target: Some(format!("{}={}", selector.attribute, selector.value)),
        });
    }
    findings
}

fn unique_findings(
    duplicates: &[DuplicateSelector],
    unique_test_ids: bool,
    unique_html_ids: bool,
) -> Vec<RuleFinding> {
    duplicates
        .iter()
        .filter_map(|selector| {
            let rule = if selector.attribute == HTML_ID_ATTRIBUTE {
                unique_html_ids.then_some(PLAYWRIGHT_UNIQUE_HTML_IDS)
            } else {
                unique_test_ids.then_some(PLAYWRIGHT_UNIQUE_TEST_IDS)
            }?;
            Some(RuleFinding {
                rule: rule.to_string(),
                file: selector.file.clone(),
                line: 1,
                message: format!(
                    "selector [{}=\"{}\"] is duplicated; Playwright selectors must be unique",
                    selector.attribute, selector.value
                ),
                import: None,
                target: Some(format!("{}={}", selector.attribute, selector.value)),
            })
        })
        .collect()
}
