use super::*;
use oxc_ast_visit::{walk, Visit};
use oxc_span::Span;
use std::path::{Path, PathBuf};

mod config_parsers;
mod parser_coverage;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/integration-tests")
            .join(name)
            .join("fixture"),
    )
}

fn fixture_file(name: &str, file: &str) -> PathBuf {
    fixture(name).join(file)
}

fn config_snippet(name: &str) -> crate::config::v2::schema::NoMistakesConfig {
    let yaml = std::fs::read_to_string(fixture_file("config-snippets", name)).unwrap();
    serde_yaml::from_str(&yaml).unwrap()
}

fn parse_vitest_fixture(
    source: &str,
    path: &Path,
    root: &Path,
) -> anyhow::Result<Vec<types::ConfigProject>> {
    let tsconfig = test_support::tsconfig_without_config(root);
    test_support::parse_vitest(source, path, root, root, &tsconfig)
}

fn parse_playwright_fixture(
    source: &str,
    path: &Path,
    config_dir: &Path,
) -> anyhow::Result<test_config::playwright::ParsedPlaywrightConfig> {
    let tsconfig = test_support::tsconfig_without_config(config_dir);
    test_support::parse_playwright(source, path, config_dir, &tsconfig)
}

#[test]
fn check_reports_integration_policy_violations() {
    let findings = check(&fixture("basic"), None).unwrap();
    let messages: Vec<_> = findings
        .iter()
        .map(|finding| {
            (
                finding.framework.as_str(),
                finding.suite.as_str(),
                finding.file.as_str(),
                finding.test_name.as_deref(),
                finding.integration.as_deref(),
            )
        })
        .collect();

    assert!(messages.contains(&(
        "vitest",
        "unit.unit",
        "backend/unit.test.mts",
        Some("direct integration in unit suite"),
        Some("openai"),
    )));
    assert!(messages.contains(&(
        "vitest",
        "unit.unit",
        "backend/unit.test.mts",
        Some("helper integration in unit suite"),
        Some("openai"),
    )));
    assert!(messages.contains(&(
        "vitest",
        "unit.unit",
        "backend/unit.test.mts",
        Some("expression helper integration in unit suite"),
        Some("openai"),
    )));
    assert!(messages.contains(&(
        "vitest",
        "mixed.openai",
        "mixed/mixed.test.mts",
        Some("wrong integration still fails in non-strict suite"),
        Some("anthropic"),
    )));
    assert!(messages.contains(&(
        "vitest",
        "mixed.openai",
        "mixed/mixed.test.mts",
        Some("wrong integration fails even when allowed integration is also called"),
        Some("anthropic"),
    )));
    assert!(messages.contains(&(
        "playwright",
        "pw-unit.unit",
        "playwright/unit/unit.spec.ts",
        Some("playwright helper integration in unit suite"),
        Some("openai"),
    )));
    assert_eq!(findings.len(), 6);
}

#[test]
fn runner_configs_share_one_parse_with_standalone_and_aggregate_source_analysis() {
    let source = fixture("parse-sharing");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible_paths = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);
    let config =
        crate::config::v2::load_v2_config_from_visible(&root, None, &visible_paths).unwrap();
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, &visible_paths)
            .unwrap();
    let test_file = root.join("src/unit.test.ts");
    let helper_file = root.join("vitest.projects.ts");
    let vitest_config_file = root.join("vitest.config.ts");
    let playwright_config_file = root.join("playwright.config.ts");

    crate::ast::begin_parse_count(&root);
    let standalone = check(&root, None).unwrap();
    let standalone_counts = crate::ast::finish_parse_count(&root);

    let runner_configs = runner_config::prepare_with_catalog_and_sources(
        &root,
        &config,
        &visible_paths,
        std::sync::Arc::new(crate::codebase::ts_resolver::TsConfigCatalog::forced(
            &root,
            tsconfig.clone(),
            None,
        )),
        std::sync::Arc::clone(&sources),
    );
    let playwright_settings =
        crate::playwright::config::test_support::load_settings(&root, None, &[], None).unwrap();
    let mut playwright_plan = crate::codebase::check_facts::PlaywrightFactPlan::from_settings(
        &root,
        playwright_settings,
        std::collections::HashMap::new(),
        false,
        &snapshot,
    )
    .unwrap();
    playwright_plan.set_source_files(vec![test_file.clone(), helper_file.clone()]);
    playwright_plan.set_app_source_files([helper_file.clone()]);
    crate::ast::begin_parse_count(&root);
    let shared =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            vec![test_file.clone(), helper_file.clone()],
            vec![vitest_config_file.clone(), playwright_config_file.clone()],
            crate::codebase::check_facts::CheckFactPlan {
                imports: true,
                symbols: true,
                integration: true,
                integration_runner_configs: Some(std::sync::Arc::new(runner_configs)),
                graph: crate::codebase::ts_source::facts::TsFactPlan::imports(),
                ..Default::default()
            },
            Some(playwright_plan),
            std::sync::Arc::clone(&sources),
        );
    let aggregate =
        check_with_prepared_facts(&root, &config, &shared, &tsconfig, &snapshot).unwrap();
    let aggregate_counts = crate::ast::finish_parse_count(&root);

    assert_eq!(aggregate, standalone);
    assert_eq!(standalone.len(), 1);
    assert_eq!(standalone[0].integration.as_deref(), Some("openai"));
    assert_eq!(standalone_counts.len(), 4, "{standalone_counts:?}");
    assert!(
        standalone_counts.values().all(|count| *count == 1),
        "{standalone_counts:?}"
    );
    assert_eq!(aggregate_counts.len(), 4, "{aggregate_counts:?}");
    assert!(
        aggregate_counts.values().all(|count| *count == 1),
        "{aggregate_counts:?}"
    );
    assert_eq!(standalone_counts.get(&test_file), Some(&1));
    assert_eq!(standalone_counts.get(&helper_file), Some(&1));
    assert_eq!(standalone_counts.get(&vitest_config_file), Some(&1));
    assert_eq!(standalone_counts.get(&playwright_config_file), Some(&1));
    assert_eq!(aggregate_counts.get(&test_file), Some(&1));
    assert_eq!(aggregate_counts.get(&helper_file), Some(&1));
    assert_eq!(aggregate_counts.get(&vitest_config_file), Some(&1));
    assert_eq!(aggregate_counts.get(&playwright_config_file), Some(&1));
    for path in [
        &test_file,
        &helper_file,
        &vitest_config_file,
        &playwright_config_file,
    ] {
        let before = sources.physical_read_count();
        let _ = sources.read_path(path);
        assert_eq!(
            sources.physical_read_count(),
            before,
            "missing source-store read: {}",
            path.display(),
        );
    }
    assert_eq!(sources.physical_read_count(), 4);
}

#[test]
fn prepared_integration_check_rejects_incomplete_runner_facts_without_reparsing() {
    let root = fixture("parse-sharing");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible_paths = snapshot.paths_for(&root);
    let config =
        crate::config::v2::load_v2_config_from_visible(&root, None, &visible_paths).unwrap();
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, &visible_paths)
            .unwrap();

    let error = check_with_prepared_facts(
        &root,
        &config,
        &crate::codebase::check_facts::CheckFactMap::default(),
        &tsconfig,
        &snapshot,
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("prepared integration runner facts are incomplete"),
        "{error:#}"
    );

    let source = include_str!("../integration_tests.rs");
    let prepared_body = source
        .split("pub fn check_with_prepared_facts(")
        .nth(1)
        .and_then(|body| body.split("fn fail_on_dropped_files(").next())
        .expect("prepared integration entrypoint body");
    assert!(!prepared_body.contains("parse_all("));
}

#[test]
fn multiple_integration_suites_for_one_project_share_project_scope_once() {
    let root = fixture("basic");
    let config = fixture_file("basic", "multiple-integration-suites.no-mistakes.yml");
    let findings = check(&root, Some(&config)).unwrap();

    assert_eq!(findings, Vec::new());
}

#[test]
fn empty_project_policy_is_allowed() {
    let config = config_snippet("empty-project-policy.yml");
    config::validate_config(&config).unwrap();
}

#[test]
fn invalid_empty_integration_suites_is_rejected() {
    let config = config_snippet("invalid-empty-integration-suites.yml");
    let err = config::validate_config(&config).unwrap_err();
    assert!(err
        .to_string()
        .contains("tests.vitest.projects.web.integration_suites.openai"));
}

#[test]
fn test_project_exclude_requires_include() {
    let config: crate::config::v2::schema::NoMistakesConfig = serde_yaml::from_str(
        r#"
tests:
  vitest:
    projects:
      web:
        exclude: ["web/generated/**"]
"#,
    )
    .unwrap();
    let err = config::validate_config(&config).unwrap_err();
    assert!(err
        .to_string()
        .contains("tests.vitest.projects.web.exclude requires include"));
}

#[test]
fn annotation_requires_one_valid_value() {
    let valid = "const f = /* no-mistakes: integration=openai */ async () => {}";
    let valid_start = valid.find("async").unwrap() as u32;
    assert_eq!(
        calls::integration_annotation_before(valid, Span::new(valid_start, valid_start + 5))
            .as_deref(),
        Some("openai")
    );

    let jsdoc = "/**\n * no-mistakes: integration: aws\n */\nasync function f() {}";
    let jsdoc_start = jsdoc.find("async").unwrap() as u32;
    assert_eq!(
        calls::integration_annotation_before(jsdoc, Span::new(jsdoc_start, jsdoc_start + 5))
            .as_deref(),
        Some("aws")
    );

    let invalid = "const f = /* no-mistakes: integration=openai,anthropic */ async () => {}";
    let invalid_start = invalid.find("async").unwrap() as u32;
    assert!(calls::integration_annotation_before(
        invalid,
        Span::new(invalid_start, invalid_start + 5)
    )
    .is_none());
}

#[test]
fn conditional_vitest_wrappers_are_detected_as_tests() {
    let source = "it.skipIf(!process.env.OPENAI_API_KEY)('real openai', async () => {})";
    crate::ast::with_program(Path::new("conditional.test.mts"), source, |program, _| {
        let mut names = Vec::new();
        struct Collector<'a>(&'a mut Vec<String>);
        impl<'a> Visit<'a> for Collector<'_> {
            fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
                if let Some(name) = calls::test_name(call) {
                    self.0.push(name);
                }
                walk::walk_call_expression(self, call);
            }
        }
        Collector(&mut names).visit_program(program);
        assert_eq!(names, vec!["real openai"]);
    })
    .unwrap();
}

mod configured_suites;
