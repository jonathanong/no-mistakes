use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope, RuleTestTargets, StringOrList};
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

fn config_with_targeted_rules(rules: Vec<(&str, Vec<&str>)>) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: rules
            .into_iter()
            .map(|(rule, targets)| RuleDef {
                rule: rule.to_string(),
                scope: Some(RuleScope::Repository),
                tests: RuleTestTargets {
                    playwright: targets.into_iter().map(str::to_string).collect(),
                    ..RuleTestTargets::default()
                },
                ..RuleDef::default()
            })
            .collect(),
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
fn check_unique_rules_do_not_require_nextjs_routes() {
    let root = fixture_path(&["scan-config", "json"]);
    let config = config_with_rule(PLAYWRIGHT_UNIQUE_TEST_IDS);

    let findings = check(&root, None, &config).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn check_with_facts_falls_back_when_shared_playwright_facts_are_missing() {
    let root = fixture_path(&["nextjs-coverage", "covered"]);
    let config = config_with_rule(PLAYWRIGHT_COVERAGE);
    let facts = CheckFactMap::default();

    let findings = check_with_facts(&root, None, &config, &facts).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn rule_selections_merge_rules_by_playwright_target() {
    let config = config_with_targeted_rules(vec![
        (PLAYWRIGHT_COVERAGE, vec!["web"]),
        (PLAYWRIGHT_UNIQUE_TEST_IDS, vec!["web", "storybook"]),
        (PLAYWRIGHT_UNIQUE_HTML_IDS, vec!["storybook"]),
    ]);

    let selections = rule_selections(&config);

    assert_eq!(selections.len(), 2);
    let web = selections
        .iter()
        .find(|selection| selection.project.as_deref() == Some("web"))
        .expect("web selection");
    assert!(web.coverage);
    assert!(web.unique_test_ids);
    assert!(!web.unique_html_ids);
    let storybook = selections
        .iter()
        .find(|selection| selection.project.as_deref() == Some("storybook"))
        .expect("storybook selection");
    assert!(!storybook.coverage);
    assert!(storybook.unique_test_ids);
    assert!(storybook.unique_html_ids);
}

#[test]
fn rule_selections_keep_unscoped_rules_global() {
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: PLAYWRIGHT_COVERAGE.to_string(),
            scope: Some(RuleScope::Repository),
            ..RuleDef::default()
        }],
        ..NoMistakesConfig::default()
    };

    let selections = rule_selections(&config);

    assert_eq!(selections.len(), 1);
    assert!(selections[0].project.is_none());
    assert!(selections[0].coverage);
}

#[test]
fn fact_plan_validates_targeted_playwright_config_names() {
    let root = fixture_path(&["playwright-configs", "multi-config"]);
    let mut config = config_with_targeted_rules(vec![(PLAYWRIGHT_COVERAGE, vec!["missing"])]);
    config.tests.playwright.configs = Some(StringOrList::Many(vec![
        "playwright.config.mts".to_string(),
        "playwright.storybook.config.mts".to_string(),
    ]));

    let error = match fact_plan(&root, None, &config) {
        Ok(_) => panic!("expected targeted Playwright config validation error"),
        Err(error) => error,
    };

    assert!(error
        .to_string()
        .contains("no Playwright config found with name missing"));
}
