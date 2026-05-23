use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::playwright::test_support::fixture_path;

fn config_with_rule(rule: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: rule.to_string(),
            scope: Some(RuleScope::Repository),
            ..RuleDef::default()
        }],
        ..NoMistakesConfig::default()
    }
}

#[test]
fn configured_is_false_without_playwright_rules() {
    let config = NoMistakesConfig::default();

    assert!(!configured(&config));
    assert!(fact_plan(
        &fixture_path(&["nextjs-coverage", "covered"]),
        None,
        &config
    )
    .unwrap()
    .is_none());
    assert!(check(
        &fixture_path(&["nextjs-coverage", "covered"]),
        None,
        &config
    )
    .unwrap()
    .is_empty());
}

#[test]
fn check_reports_coverage_without_shared_facts() {
    let root = fixture_path(&["nextjs-coverage", "uncovered"]);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();

    let findings = check(&root, None, &config).unwrap();

    assert!(findings
        .iter()
        .any(|finding| finding.rule == PLAYWRIGHT_COVERAGE));
}

#[test]
fn check_with_facts_returns_empty_when_disabled() {
    let root = fixture_path(&["nextjs-coverage", "covered"]);
    let config = NoMistakesConfig::default();
    let facts = CheckFactMap::default();

    let findings = check_with_facts(&root, None, &config, &facts).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn check_reports_analysis_errors_without_shared_facts() {
    let root = fixture_path(&["scan-config", "json"]);
    let config = config_with_rule(PLAYWRIGHT_COVERAGE);

    let error = check(&root, None, &config).unwrap_err();

    assert!(error.to_string().contains("no Next.js page routes found"));
}

#[test]
fn check_with_facts_reports_missing_shared_playwright_facts() {
    let root = fixture_path(&["nextjs-coverage", "covered"]);
    let config = config_with_rule(PLAYWRIGHT_COVERAGE);
    let facts = CheckFactMap::default();

    let error = check_with_facts(&root, None, &config, &facts).unwrap_err();

    assert!(error
        .to_string()
        .contains("missing shared Playwright facts"));
}
