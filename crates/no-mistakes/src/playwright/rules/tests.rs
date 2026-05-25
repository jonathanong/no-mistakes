use super::*;
use crate::codebase::rules::RuleFinding;
use crate::config::v2::schema::{RuleDef, RuleScope, RuleTestTargets, StringOrList};
use crate::playwright::test_support::fixture_path;
use std::fs;

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
fn check_with_facts_unique_rules_do_not_require_nextjs_routes() {
    let root = fixture_path(&["scan-config", "json"]);
    let config = config_with_rule(PLAYWRIGHT_UNIQUE_TEST_IDS);
    let facts = CheckFactMap::default();

    let findings = check_with_facts(&root, None, &config, &facts).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn check_with_facts_unique_rules_propagate_analysis_errors() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let config_path = root.join(".no-mistakes.yml");
    fs::write(
        &config_path,
        "tests:\n  playwright:\n    configs: missing.config.ts\n",
    )
    .unwrap();
    let config = config_with_rule(PLAYWRIGHT_UNIQUE_TEST_IDS);
    let facts = CheckFactMap::default();

    let error = check_with_facts(root, Some(&config_path), &config, &facts).unwrap_err();

    assert!(error
        .to_string()
        .contains("Playwright config does not exist"));
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
        .find(|selection| selection.playwright_project.as_deref() == Some("web"))
        .expect("web selection");
    assert!(web.coverage);
    assert!(web.unique_test_ids);
    assert!(!web.unique_html_ids);
    let storybook = selections
        .iter()
        .find(|selection| selection.playwright_project.as_deref() == Some("storybook"))
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
    assert!(selections[0].playwright_project.is_none());
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

#[test]
fn fact_plan_merges_test_id_attributes_for_shared_target_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(
        root.join("tests/shared.spec.ts"),
        "import { test } from '@playwright/test'; test('shared', async ({ page }) => { await page.getByTestId('shared').click(); });",
    )
    .unwrap();
    fs::write(
        root.join("playwright.a.config.ts"),
        "export default { name: 'a', testDir: './tests', use: { testIdAttribute: 'data-a' } };",
    )
    .unwrap();
    fs::write(
        root.join("playwright.b.config.ts"),
        "export default { name: 'b', testDir: './tests', use: { testIdAttribute: 'data-b' } };",
    )
    .unwrap();
    let mut config = config_with_targeted_rules(vec![
        (PLAYWRIGHT_UNIQUE_TEST_IDS, vec!["a"]),
        (PLAYWRIGHT_UNIQUE_HTML_IDS, vec!["b"]),
    ]);
    config.tests.playwright.configs = Some(StringOrList::Many(vec![
        "playwright.a.config.ts".to_string(),
        "playwright.b.config.ts".to_string(),
    ]));

    let plan = fact_plan(root, None, &config).unwrap().unwrap();
    let attributes = plan
        .test_id_attributes_by_path
        .get(&root.join("tests/shared.spec.ts"))
        .expect("shared test file attributes");

    assert_eq!(
        attributes,
        &vec!["data-a".to_string(), "data-b".to_string()]
    );
}

#[test]
fn filter_rule_findings_applies_path_filters_per_playwright_rule() {
    let root = std::path::Path::new("/repo");
    let config = NoMistakesConfig {
        rules: vec![
            RuleDef {
                rule: PLAYWRIGHT_UNIQUE_TEST_IDS.to_string(),
                scope: Some(RuleScope::Repository),
                exclude: vec!["tests/generated/**".to_string()],
                ..RuleDef::default()
            },
            RuleDef {
                rule: PLAYWRIGHT_UNIQUE_HTML_IDS.to_string(),
                scope: Some(RuleScope::Repository),
                include: vec!["tests/pages/**".to_string()],
                ..RuleDef::default()
            },
        ],
        ..NoMistakesConfig::default()
    };
    let findings = vec![
        RuleFinding {
            rule: PLAYWRIGHT_UNIQUE_TEST_IDS.to_string(),
            file: "tests/login.spec.ts".to_string(),
            line: 1,
            message: "duplicate test id".to_string(),
            import: None,
            target: None,
        },
        RuleFinding {
            rule: PLAYWRIGHT_UNIQUE_TEST_IDS.to_string(),
            file: "tests/generated/login.spec.ts".to_string(),
            line: 1,
            message: "duplicate test id".to_string(),
            import: None,
            target: None,
        },
        RuleFinding {
            rule: PLAYWRIGHT_UNIQUE_HTML_IDS.to_string(),
            file: "tests/pages/home.spec.ts".to_string(),
            line: 1,
            message: "duplicate html id".to_string(),
            import: None,
            target: None,
        },
        RuleFinding {
            rule: PLAYWRIGHT_UNIQUE_HTML_IDS.to_string(),
            file: "tests/components/button.spec.ts".to_string(),
            line: 1,
            message: "duplicate html id".to_string(),
            import: None,
            target: None,
        },
    ];

    let filtered = filter::filter_rule_findings(root, &config, findings).unwrap();

    assert_eq!(
        filtered
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        vec!["tests/pages/home.spec.ts", "tests/login.spec.ts"]
    );
}
