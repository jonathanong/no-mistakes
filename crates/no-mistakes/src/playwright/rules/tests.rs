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

fn staged_playwright_fixture() -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/staged-playwright/fixture"),
    )
}

fn html_id_rule_composition_findings(name: &str) -> Vec<RuleFinding> {
    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/playwright/html-id-rule-composition")
        .join(name);
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();

    check(&root, None, &config).unwrap()
}

#[test]
fn html_id_uniqueness_does_not_enable_coverage_in_standalone_checks() {
    let findings = html_id_rule_composition_findings("html-disabled-unique");
    assert!(findings.is_empty(), "{findings:?}");

    let findings = html_id_rule_composition_findings("html-disabled-duplicate");
    assert!(
        findings.iter().any(|finding| {
            finding.rule == PLAYWRIGHT_UNIQUE_HTML_IDS
                && finding.target.as_deref() == Some("id=duplicate-disabled")
        }),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| finding.rule == PLAYWRIGHT_UNIQUE_HTML_IDS),
        "{findings:?}"
    );

    let findings = html_id_rule_composition_findings("html-enabled-unique");
    assert!(
        findings.iter().any(|finding| {
            finding.rule == PLAYWRIGHT_COVERAGE
                && finding.target.as_deref() == Some("id=unique-enabled")
        }),
        "{findings:?}"
    );

    for (fixture, target) in [
        ("explicit-id-test-attribute", "id=explicit-test-id"),
        ("explicit-id-component-mapping", "id=explicit-component-id"),
    ] {
        let findings = html_id_rule_composition_findings(fixture);
        assert!(
            findings.iter().any(|finding| {
                finding.rule == PLAYWRIGHT_COVERAGE && finding.target.as_deref() == Some(target)
            }),
            "{fixture}: {findings:?}"
        );
    }
}

#[test]
fn html_id_uniqueness_on_one_target_does_not_widen_another_targets_coverage() {
    let findings = html_id_rule_composition_findings("multi-project-isolation");

    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn aggregate_playwright_rule_parses_each_source_file_once() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/playwright"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let snapshot = std::sync::Arc::new(crate::playwright::fsutil::VisiblePathSnapshot::new(&root));
    let visible_paths = snapshot.paths_for(&root);
    let config =
        crate::config::v2::load_v2_config_from_visible(&root, None, &visible_paths).unwrap();
    let files = crate::codebase::ts_source::discover_files_from_visible(
        &root,
        &config.filesystem.skip_directories,
        &visible_paths,
    );

    crate::ast::begin_parse_count(&root);
    let paths = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
        None, &root, &paths, &sources,
    )
    .unwrap();
    let prepared = prepare_from_snapshot(
        &root,
        None,
        &config,
        std::sync::Arc::clone(&snapshot),
        std::sync::Arc::new(tsconfig),
    )
    .unwrap()
    .expect("fixture enables a Playwright rule");
    let facts = crate::codebase::check_facts::collect_check_facts_with_graph_files_and_playwright(
        &root,
        files,
        Vec::new(),
        crate::codebase::check_facts::CheckFactPlan::default(),
        Some(prepared.fact_plan()),
    );
    let findings = check_with_prepared_facts(&root, None, &config, &facts, &prepared).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let expected = [
        root.join("app/Widget.tsx"),
        root.join("app/page.tsx"),
        root.join("playwright.config.ts"),
        root.join("playwright.helper.ts"),
        root.join("tests/home.spec.ts"),
    ];

    assert!(
        findings.iter().all(|finding| !finding.file.is_empty()),
        "{findings:?}"
    );
    assert_eq!(counts.len(), expected.len(), "{counts:?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
    for file in expected {
        assert_eq!(counts.get(&file), Some(&1), "{counts:?}");
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

// Regression tests for #343: `getByTestId(...)` resolution when the Playwright
// config's `testIdAttribute` is hidden in a helper or differs from the configured
// `selectors.testIds`.

#[test]
fn helper_wrapped_config_falls_back_to_configured_test_ids() {
    // The Playwright config is `defineConfig(createPlaywrightConfig({...}))`, so
    // `testIdAttribute: 'data-pw'` (set inside the helper) is not readable. The
    // fallback resolves `getByTestId('save')` against the configured testIds
    // (data-pw), so `data-pw="save"` is covered and there are no findings.
    let root = fixture_path(&["nextjs-coverage", "helper-config-testid"]);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();

    let findings = check(&root, None, &config).unwrap();

    assert!(
        findings.is_empty(),
        "expected the data-pw selector to be covered via the testIds fallback, got {findings:?}"
    );
}

#[test]
fn readable_test_id_attribute_wins_over_configured_test_ids() {
    // The config statically sets `testIdAttribute: 'data-qa'`, which beats the
    // `selectors.testIds` (data-pw) fallback. `getByTestId('save')` therefore
    // resolves to data-qa and does NOT cover `data-pw="save"`.
    let root = fixture_path(&["nextjs-coverage", "readable-testid-mismatch"]);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();

    let findings = check(&root, None, &config).unwrap();

    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains(r#"[data-pw="save"]"#)),
        "expected the data-pw selector to be uncovered, got {findings:?}"
    );
}

#[test]
fn explicit_test_id_attribute_override_beats_readable_config() {
    // The fixture's `.no-mistakes.yml` sets `tests.playwright.testIdAttribute:
    // data-pw`, which overrides even the statically-read `data-qa`, restoring
    // coverage of `data-pw="save"`.
    let root = fixture_path(&["nextjs-coverage", "override-testid-attribute"]);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();

    let findings = check(&root, None, &config).unwrap();

    assert!(
        findings.is_empty(),
        "expected the testIdAttribute override to cover the selector, got {findings:?}"
    );
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
        (PLAYWRIGHT_PREFER_TEST_ID_LOCATORS, vec!["web"]),
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
    assert!(web.prefer_test_id_locators);
    let storybook = selections
        .iter()
        .find(|selection| selection.playwright_project.as_deref() == Some("storybook"))
        .expect("storybook selection");
    assert!(!storybook.coverage);
    assert!(storybook.unique_test_ids);
    assert!(storybook.unique_html_ids);
    assert!(!storybook.prefer_test_id_locators);
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
    let root = staged_playwright_fixture();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();

    let plan = fact_plan(&root, None, &config).unwrap().unwrap();
    let file_plan = plan
        .file(&root.join("tests/multi.spec.ts"))
        .expect("shared test file settings");
    let attributes = file_plan.merged_test_id_attributes();

    assert_eq!(attributes, vec!["data-a".to_string(), "data-b".to_string()]);
    assert_eq!(file_plan.selector_extraction_count(), 2);
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

#[test]
fn uncovered_selector_message_mentions_helper_wrapper_reference() {
    let root = fixture_path(&["nextjs-selectors", "helper-wrapper-reference"]);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();

    let findings = check(&root, None, &config).unwrap();
    let finding = findings
        .iter()
        .find(|finding| finding.target.as_deref() == Some("data-pw=example-button"))
        .expect("uncovered example-button selector finding");

    assert!(
        finding
            .message
            .contains("found 'example-button' in tests/e2e/app.spec.ts:9 getAsideLocator(...)"),
        "expected helper-wrapper hint in finding, got: {}",
        finding.message
    );
    assert!(finding
        .message
        .contains("selector coverage only counts literal getByTestId"));
}
