// no-mistakes-disable-file rust-max-lines-per-file: legacy parser coverage suite
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

    let runner_configs = runner_config::prepare_with_sources(
        &root,
        &config,
        &visible_paths,
        &tsconfig,
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

#[test]
fn coverage_fixture_exercises_parser_and_resolution_variants() {
    let root = fixture("coverage");
    let findings = check(&root, None).unwrap();
    assert!(findings.iter().any(|finding| {
        finding.framework == "vitest"
            && finding.suite == "root-vitest.openai"
            && finding.test_name.as_deref() == Some("uses declared function")
            && finding.integration.as_deref() == Some("openai")
    }));
    assert!(findings.iter().any(|finding| {
        finding.suite == "root-vitest.openai"
            && finding.test_name.as_deref() == Some("uses namespace function")
            && finding.integration.as_deref() == Some("openai")
    }));
    assert!(findings
        .iter()
        .all(|finding| finding.suite != "nested-suite"));
}

#[test]
fn invalid_suite_project_and_missing_config_are_rejected() {
    let missing = check(&fixture("missing-config"), None).unwrap_err();
    assert!(missing.to_string().contains("config does not exist"));

    let unknown = check(&fixture("unknown-project"), None).unwrap_err();
    assert!(unknown
        .to_string()
        .contains("vitest integration policy references unknown project missing"));
}

#[test]
fn configured_suites_cover_matching_variants() {
    let root = fixture("coverage");
    let config = config_snippet("configured-suites.yml");
    let suites = test_support::configured_suites(&root, &config).unwrap();
    assert!(suites.iter().any(|suite| suite.name == "inherits.openai"));
    assert!(suites.iter().any(|suite| suite.name == "absolute.openai"
        && suite.include == vec!["/tmp/no-mistakes-absolute-tests/**/*.spec.ts"]));
    assert!(suites
        .iter()
        .any(|suite| suite.name == "root-vitest.openai"));

    let config = config_snippet("missing-playwright-config.yml");
    let err = test_support::configured_suites(&root, &config).unwrap_err();
    assert!(err.to_string().contains("config does not exist"));

    let config = config_snippet("empty-policy-with-missing-config.yml");
    assert!(test_support::configured_suites(&root, &config)
        .unwrap()
        .is_empty());

    let config = config_snippet("mixed-empty-and-nonempty-policy.yml");
    let suites = test_support::configured_suites(&root, &config).unwrap();
    assert_eq!(suites.len(), 1);
    assert_eq!(suites[0].name, "root-vitest.openai");

    let config = config_snippet("explicit-project-policy.yml");
    let suites = test_support::configured_suites(&root, &config).unwrap();
    assert_eq!(suites.len(), 1);
    assert_eq!(suites[0].name, "explicit.openai");
    assert_eq!(suites[0].include, vec!["explicit/**/*.test.ts"]);
    assert_eq!(suites[0].exclude, vec!["explicit/**/*.mock.test.ts"]);

    assert!(
        project_config::load_projects(&root, types::Framework::Vitest, None)
            .unwrap()
            .is_empty()
    );
    let commonjs_root = fixture("cjs-cts-configs");
    assert!(
        project_config::load_projects(&commonjs_root, types::Framework::Vitest, None)
            .unwrap()
            .iter()
            .any(
                |project| project.config.as_deref() == Some("vitest.config.cts")
                    && project.policy_name.as_deref() == Some("unit")
            )
    );
    assert!(
        project_config::load_projects(&commonjs_root, types::Framework::Playwright, None)
            .unwrap()
            .iter()
            .any(|project| project.config.as_deref() == Some("playwright.config.cjs"))
    );
    assert!(
        !project_config::load_projects(&fixture("basic"), types::Framework::Playwright, None)
            .unwrap()
            .is_empty()
    );
    let visible = crate::codebase::ts_source::VisiblePathSnapshot::new(&root).paths_for(&root);
    assert!(
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, &visible)
            .unwrap()
            .base_url
            .is_some()
    );
    assert!(project_config::build_globset(&["[".to_string()]).is_err());
    assert!(!project_config::load_projects(
        &root,
        types::Framework::Playwright,
        Some(&crate::config::v2::schema::StringOrList::One(
            "playwright.projects.ts".to_string()
        )),
    )
    .unwrap()
    .is_empty());
    assert!(project_config::load_projects(
        &root,
        types::Framework::Playwright,
        Some(&crate::config::v2::schema::StringOrList::One(
            "playwright.invalid.ts".to_string()
        )),
    )
    .is_err());
    let package_root = fixture("vitest-package-tsconfig");
    let package_snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&package_root);
    let package_config_dir = package_root.join("packages/app");
    let package_tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
        None,
        &package_config_dir,
        &package_snapshot.paths_for(&package_config_dir),
    )
    .unwrap();
    let package_projects = project_config::load_projects_from_visible(
        &package_root,
        types::Framework::Vitest,
        Some(&crate::config::v2::schema::StringOrList::One(
            "packages/app/vitest.config.mts".to_string(),
        )),
        &package_snapshot.paths_for(&package_root),
        &package_tsconfig,
    )
    .unwrap();
    assert!(package_projects.iter().any(|project| {
        project.config.as_deref() == Some("packages/app/vitest.config.mts")
            && project.policy_name.as_deref() == Some("package")
            && project.include == vec!["packages/app/package/**/*.test.ts"]
    }));
    let invalid_tsconfig_root = fixture("invalid-vitest-tsconfig");
    let err = project_config::load_projects(
        &invalid_tsconfig_root,
        types::Framework::Vitest,
        Some(&crate::config::v2::schema::StringOrList::One(
            "vitest.config.mts".to_string(),
        )),
    )
    .unwrap_err();
    assert!(format!("{err:#}").contains("loading tsconfig"));
    let invalid_playwright_tsconfig_root = fixture("playwright-invalid-tsconfig");
    let playwright_err = project_config::load_projects(
        &invalid_playwright_tsconfig_root,
        types::Framework::Playwright,
        Some(&crate::config::v2::schema::StringOrList::One(
            "playwright.config.ts".to_string(),
        )),
    )
    .unwrap_err();
    assert!(format!("{playwright_err:#}").contains("loading tsconfig"));

    let config = config_snippet("missing-config-and-project.yml");
    let err = test_support::configured_suites(&root, &config).unwrap_err();
    assert!(err.to_string().contains("config does not exist"));
}

#[test]
fn configured_suites_reject_duplicate_project_names() {
    let root = fixture("duplicate-projects");
    let config = config_snippet("duplicate-vitest-project-policy.yml");

    let err = test_support::configured_suites(&root, &config).unwrap_err();

    assert!(err
        .to_string()
        .contains("vitest integration policy references ambiguous project unit"));
}

#[test]
fn configured_suites_support_vitest_commonjs_auto_discovery() {
    let root = fixture("vitest-cjs-config");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();

    let suites = test_support::configured_suites(&root, &config).unwrap();

    assert_eq!(suites.len(), 1);
    assert_eq!(suites[0].name, "unit.openai");
    assert_eq!(suites[0].include, vec!["unit/**/*.test.ts"]);
}

#[test]
fn analyze_files_covers_import_and_function_shapes() {
    let file = fixture_file("coverage", "src/source.test.ts");
    let missing = fixture_file("coverage", "src/does-not-exist.ts");
    let analyses = test_support::analyze_files(&[missing, file.clone()]).unwrap();
    let analysis = analyses.get(&file).unwrap();

    assert!(analysis.imports.contains_key("defaultCall"));
    assert!(analysis.imports.contains_key("renamedCall"));
    assert!(analysis.imports.contains_key("helperNamespace"));
    assert!(analysis.functions.contains_key("declaredIntegration"));
    assert!(analysis.functions.contains_key("arrowIntegration"));
    assert!(analysis.functions.contains_key("functionIntegration"));
    assert!(analysis.functions.contains_key("exportedDeclared"));
    assert!(analysis.functions.contains_key("exportedArrow"));
    assert!(analysis.functions.contains_key("exportedFunction"));
    assert!(analysis
        .tests
        .iter()
        .any(|test| test.name.as_deref() == Some("uses declared function")));
}

#[test]
fn call_helpers_cover_non_test_and_member_variants() {
    let path = fixture_file("coverage", "src/calls.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let mut collector = CallAssertions::default();
        collector.visit_program(program);
        assert!(collector.saw_describe_as_non_test);
        assert!(collector.saw_non_string_test);
        assert!(collector.saw_function_callback);
        assert!(collector.saw_imported_member_call);
        assert!(collector.saw_non_callback_argument);
    })
    .unwrap();
}

#[derive(Default)]
struct CallAssertions {
    saw_describe_as_non_test: bool,
    saw_non_string_test: bool,
    saw_function_callback: bool,
    saw_imported_member_call: bool,
    saw_non_callback_argument: bool,
}

impl<'a> Visit<'a> for CallAssertions {
    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        let path = crate::ast::expression_path(&call.callee);
        if path
            .as_ref()
            .is_some_and(|path| path == &["test", "describe"])
        {
            self.saw_describe_as_non_test = calls::test_name(call).is_none();
        }
        if path.as_ref().is_some_and(|path| path == &["test"]) && calls::test_name(call).is_none() {
            self.saw_non_string_test = true;
            self.saw_non_callback_argument = calls::callback_argument(call).is_none();
            assert!(calls::collect_calls(call.arguments.first().unwrap()).is_empty());
        }
        if calls::test_name(call).as_deref() == Some("function callback") {
            let (argument, _) = calls::callback_argument(call).unwrap();
            let collected = calls::collect_calls(argument);
            self.saw_function_callback = true;
            self.saw_imported_member_call = collected.iter().any(
                |target| matches!(target, types::CallTarget::Imported { local } if local == "foo"),
            );
        }
        walk::walk_call_expression(self, call);
    }
}

#[test]
fn swift_framework_string_is_stable() {
    assert_eq!(types::Framework::Dotnet.as_str(), "dotnet");
    assert_eq!(types::Framework::Swift.as_str(), "swift");
}
