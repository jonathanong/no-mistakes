use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};

mod extended;
mod gitignore_tsconfig;

fn fixture(path: &str) -> std::path::PathBuf {
    let mut parts = path.splitn(3, '/');
    let category = parts.next().unwrap_or(path);
    let sub = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("");
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases")
        .join(category)
        .join(sub)
        .join("fixture");
    if !rest.is_empty() {
        p = p.join(rest);
    }
    crate::codebase::ts_resolver::normalize_path(&p)
}

#[test]
fn rule_enabled_requires_configured_rule() {
    let mut config = crate::config::v2::NoMistakesConfig::default();
    assert!(!rule_enabled(&config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS));
    config.rules.push(RuleDef {
        rule: TEST_NO_UNMOCKED_DYNAMIC_IMPORTS.to_string(),
        scope: Some(RuleScope::Repository),
        ..Default::default()
    });
    assert!(rule_enabled(&config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS));
}

#[test]
fn rule_enabled_accepts_project_rule_without_top_level_options() {
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.projects.insert(
        "tests".to_string(),
        crate::config::v2::schema::Project::default(),
    );
    config.rules.push(RuleDef {
        rule: TEST_NO_UNMOCKED_DYNAMIC_IMPORTS.to_string(),
        projects: vec!["tests".to_string()],
        ..Default::default()
    });
    assert!(rule_enabled(&config, TEST_NO_UNMOCKED_DYNAMIC_IMPORTS));
}

#[test]
fn run_check_returns_empty_when_rule_is_not_enabled() {
    let root = std::path::Path::new("/tmp/no-mistakes-empty-rules");
    let findings = run_check(root, None, None).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn run_check_executes_enabled_rule() {
    let root = fixture("codebase-analysis/test-no-unmocked-dynamic-imports");
    let findings = run_check(&root, None, None).unwrap();
    assert!(findings
        .iter()
        .any(|finding| finding.target.as_deref() == Some("src/unmocked-child.mts")));
}

#[test]
fn run_check_executes_storybook_rule() {
    let root = fixture("rules/require-storybook-stories/covered");
    let findings = run_check(&root, None, None).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn run_check_executes_playwright_rules() {
    let root = fixture("check-runner/playwright-unique-test-ids");
    let findings = run_check(&root, None, None).unwrap();
    assert!(findings
        .iter()
        .any(|finding| finding.rule == crate::playwright::rules::PLAYWRIGHT_UNIQUE_TEST_IDS));
}

#[test]
fn run_check_with_facts_executes_playwright_rules() {
    let root = fixture("check-runner/playwright-unique-test-ids");
    let facts = crate::codebase::check_facts::CheckFactMap::default();
    let findings = run_check_with_facts(&root, None, None, &facts).unwrap();
    assert!(findings
        .iter()
        .any(|finding| finding.rule == crate::playwright::rules::PLAYWRIGHT_UNIQUE_TEST_IDS));
}

#[test]
fn run_check_with_facts_propagates_playwright_rule_errors() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let config_path = root.join(".no-mistakes.yml");
    std::fs::write(
        &config_path,
        "tests:\n  playwright:\n    configs: missing.config.ts\nrules:\n  - rule: playwright-unique-test-ids\n    scope: repository\n",
    )
    .unwrap();
    let facts = crate::codebase::check_facts::CheckFactMap::default();
    let error = run_check_with_facts(root, Some(&config_path), None, &facts).unwrap_err();
    assert!(error
        .to_string()
        .contains("Playwright config does not exist"));
}

#[test]
fn run_check_with_facts_executes_storybook_rule() {
    let root = fixture("rules/require-storybook-stories/covered");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            symbols: true,
            react: true,
            storybook: true,
            source: true,
            dynamic_imports: true,
            ..Default::default()
        },
    );
    let findings = run_check_with_facts(&root, None, None, &facts).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn run_check_executes_forbidden_dependencies_rule() {
    let root = fixture("codebase-analysis/forbidden-dependencies-basic");
    let findings = run_check(&root, None, None).unwrap();
    assert!(findings.iter().any(|f| f.rule == FORBIDDEN_DEPENDENCIES));
}

#[test]
fn prepared_run_check_matches_the_individual_forbidden_dependency_api() {
    let root = fixture("codebase-analysis/forbidden-dependencies-basic");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let tsconfig = root.join("tsconfig.json");
    let prepared = run_check(&root, None, Some(&tsconfig))
        .unwrap()
        .into_iter()
        .filter(|finding| finding.rule == FORBIDDEN_DEPENDENCIES)
        .collect::<Vec<_>>();
    let individual =
        forbidden_dependencies::check_with_config(&root, &config, None, Some(&tsconfig)).unwrap();
    assert_eq!(prepared, individual);
}

#[test]
fn standalone_rules_prepare_one_request_without_nested_rule_discovery() {
    let source = include_str!("run/standalone.rs");
    assert_eq!(source.matches("VisiblePathSnapshot::new(root)").count(), 1);
    assert_eq!(source.matches("load_v2_config_from_visible(").count(), 1);
    assert_eq!(source.matches("InferredRoots::from_visible(").count(), 1);
    assert_eq!(source.matches("resolve_tsconfig_from_visible(").count(), 1);
    assert_eq!(source.matches("prepare_from_snapshot(").count(), 1);
    assert_eq!(source.matches("prepare_graph_config(").count(), 1);
    assert_eq!(source.matches("standalone_fact_plan(&config)").count(), 1);
    assert_eq!(
        source
            .matches("collect_check_facts_with_graph_files_and_playwright(")
            .count(),
        1
    );
    assert_eq!(
        source
            .matches("run_check_with_config_and_facts_and_playwright(")
            .count(),
        1
    );
    for nested in [
        "test_no_unmocked_dynamic_imports::check(",
        "server_route_client_boundary::check(",
        "nextjs_no_api_routes::check(",
        "nextjs_no_caching::check(",
        "require_storybook_stories::check(",
        "playwright::rules::check(",
        "forbidden_dependencies::check_with_config(",
    ] {
        assert!(
            !source.contains(nested),
            "standalone fanout called {nested}"
        );
    }
}

#[test]
fn run_check_surfaces_forbidden_dependencies_tsconfig_error() {
    let root = fixture("codebase-analysis/forbidden-dependencies-basic");
    let missing_tsconfig = root.join("does-not-exist.tsconfig.json");
    let error = run_check(&root, None, Some(&missing_tsconfig)).unwrap_err();
    assert!(format!("{error:#}").contains("does-not-exist.tsconfig.json"));
}

#[test]
fn run_check_with_facts_executes_forbidden_dependencies_rule() {
    let root = fixture("codebase-analysis/forbidden-dependencies-basic");
    let tsconfig = root.join("tsconfig.json");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let graph_plan = forbidden_dependencies::graph_plan(&config).unwrap();
    let (graph, graph_context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan(&root, graph_plan);
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        crate::codebase::ts_source::discover_files(&root, &[]),
        crate::codebase::check_facts::CheckFactPlan {
            graph,
            graph_context,
            ..Default::default()
        },
    );
    let findings = run_check_with_facts(&root, None, Some(&tsconfig), &shared).unwrap();
    assert!(findings.iter().any(|f| f.rule == FORBIDDEN_DEPENDENCIES));
}

#[test]
fn run_check_with_facts_reports_missing_forbidden_dependency_graph_facts() {
    let root = fixture("codebase-analysis/forbidden-dependencies-basic");
    let shared = crate::codebase::check_facts::CheckFactMap::default();
    let error = run_check_with_facts(&root, None, None, &shared).unwrap_err();
    assert!(format!("{error:#}").contains("missing graph facts"));
}

#[test]
fn aggregate_check_propagates_server_route_missing_source_error() {
    let root = fixture("rules/server-route-client-boundary/fail");
    let config = root.join("boundary-only.no-mistakes.yml");
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        crate::codebase::ts_source::discover_files(&root, &[]),
        crate::codebase::check_facts::CheckFactPlan::default(),
    );

    let error = run_check_with_facts(&root, Some(&config), None, &shared).unwrap_err();

    assert_eq!(
        error.to_string(),
        format!(
            "server-route-client-boundary requires source facts for {}",
            root.join("backend/api/client.ts").display()
        )
    );
}

#[test]
fn aggregate_check_propagates_nextjs_api_route_missing_source_error() {
    let root = fixture("codebase-analysis/no-nextjs-api-routes");
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        crate::codebase::ts_source::discover_files(&root, &[]),
        crate::codebase::check_facts::CheckFactPlan::default(),
    );

    let error = run_check_with_facts(&root, None, None, &shared).unwrap_err();

    assert_eq!(
        error.to_string(),
        format!(
            "nextjs-no-api-routes requires source facts for {}",
            root.join("web/app/api/disabled/route.ts").display()
        )
    );
}

#[test]
fn aggregate_check_propagates_nextjs_caching_missing_source_error() {
    let root = fixture("codebase-analysis/no-nextjs-caching");
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        crate::codebase::ts_source::discover_files(&root, &[]),
        crate::codebase::check_facts::CheckFactPlan::default(),
    );

    let error = run_check_with_facts(&root, None, None, &shared).unwrap_err();

    assert_eq!(
        error.to_string(),
        format!(
            "nextjs-no-caching requires source facts for {}",
            root.join("web/app/bad.ts").display()
        )
    );
}
