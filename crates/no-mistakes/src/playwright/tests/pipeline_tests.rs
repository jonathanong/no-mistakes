use crate::playwright::analysis::app_collect::collect_app_selector_occurrences_from_visible;
use crate::playwright::analysis::cli_run::run;
use crate::playwright::analysis::context;
use crate::playwright::analysis::context::{DiscoveredTestFile, TestAnalysisContext};
use crate::playwright::analysis::output::{
    build_related_report, print_edges_text, print_related_text,
};
use crate::playwright::analysis::pipeline::analyze_with_policy_and_facts_from_snapshot;
use crate::playwright::analysis::pipeline::test_support::analyze_with_policy;
use crate::playwright::analysis::test_file::analyze_test_occurrences;
use crate::playwright::analysis::types::{Analysis, UniqueSelectorPolicy};
use crate::playwright::cli::{Command, PlaywrightArgs as Cli};
use crate::playwright::config::Settings;
use crate::playwright::fsutil::VisiblePathSnapshot;
use crate::playwright::playwright_tests;
use crate::playwright::selectors;
use crate::playwright::test_support::fixture_path;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn analyze(root: &Path, settings: &Settings) -> Result<Analysis> {
    analyze_with_policy(
        root,
        settings,
        playwright_tests::TestPolicy::default(),
        UniqueSelectorPolicy::default(),
    )
}

fn collect_app_selectors(
    root: &Path,
    settings: &Settings,
    selector_regexes: &selectors::SelectorRegexes,
) -> Result<Vec<selectors::AppSelector>> {
    let snapshot = VisiblePathSnapshot::new(root);
    let mut app_selectors =
        collect_app_selector_occurrences_from_visible(root, settings, selector_regexes, &snapshot)?;
    app_selectors.sort();
    app_selectors.dedup();
    Ok(app_selectors)
}

#[test]
fn analyze_with_facts_falls_back_when_shared_facts_are_missing() {
    let root = fixture_path(&["nextjs-coverage", "covered"]);
    let settings = Settings {
        frontend_root: "web/app".to_string(),
        playwright_configs: vec![],
        project: None,
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![],
        navigation_helpers: vec![],
        selector_wrappers: vec![],
        selector_attributes: vec!["data-testid".to_string()],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec!["web/app".to_string()],
        selector_include: vec![],
        selector_exclude: vec![],
    };
    let facts = crate::codebase::check_facts::CheckFactMap::default();
    let snapshot = VisiblePathSnapshot::new(&root);

    let analysis = analyze_with_policy_and_facts_from_snapshot(
        &root,
        &settings,
        playwright_tests::TestPolicy::default(),
        UniqueSelectorPolicy::default(),
        &facts,
        &snapshot,
    )
    .unwrap();

    assert!(!analysis.edges.edges.is_empty());
}

#[test]
fn analyze_with_facts_falls_back_when_file_facts_do_not_include_playwright() {
    let root = fixture_path(&["nextjs-coverage", "covered"]);
    let settings = Settings {
        frontend_root: "web/app".to_string(),
        playwright_configs: vec![],
        project: None,
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![],
        navigation_helpers: vec![],
        selector_wrappers: vec![],
        selector_attributes: vec!["data-testid".to_string()],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec!["web/app".to_string()],
        selector_include: vec![],
        selector_exclude: vec![],
    };
    let mut facts = crate::codebase::check_facts::CheckFactMap::default();
    facts.ts.insert(
        root.join("tests/e2e/settings.spec.ts"),
        crate::codebase::check_facts::CheckFileFacts::default().into(),
    );
    facts.ts.insert(
        root.join("tests/e2e/users.spec.ts"),
        crate::codebase::check_facts::CheckFileFacts::default().into(),
    );
    let snapshot = VisiblePathSnapshot::new(&root);

    let analysis = analyze_with_policy_and_facts_from_snapshot(
        &root,
        &settings,
        playwright_tests::TestPolicy::default(),
        UniqueSelectorPolicy::default(),
        &facts,
        &snapshot,
    )
    .unwrap();

    assert!(!analysis.edges.edges.is_empty());
}

#[test]
fn analyze_discovers_tests_and_builds_reports() {
    let root = fixture_path(&["nextjs-coverage", "covered"]);
    let settings = Settings {
        frontend_root: "web/app".to_string(),
        playwright_configs: vec![],
        project: None,
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![],
        navigation_helpers: vec![],
        selector_wrappers: vec![],
        selector_attributes: vec!["data-testid".to_string()],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec!["web/app".to_string()],
        selector_include: vec![],
        selector_exclude: vec![],
    };

    let analysis = analyze(&root, &settings).unwrap();
    assert!(!analysis.coverage.routes.is_empty());
    assert!(!analysis.edges.edges.is_empty());

    let run_root = root.join("web");
    let cli = Cli {
        root: run_root.clone(),
        config: None,
        playwright_config: vec![],
        project: None,
        json: false,
        assert_conditional_tests: false,
        allow_skipped_tests: false,
        assert_unique_test_ids: false,
        assert_unique_html_ids: false,
        command: Command::Check,
    };
    assert_eq!(run(cli.clone()).unwrap(), ExitCode::from(1));

    let mut cli_json = cli.clone();
    cli_json.json = true;
    assert_eq!(run(cli_json).unwrap(), ExitCode::from(1));

    let mut cli_edges = cli.clone();
    cli_edges.command = Command::Edges;
    assert_eq!(run(cli_edges.clone()).unwrap(), ExitCode::SUCCESS);

    let mut cli_edges_json = cli_edges;
    cli_edges_json.json = true;
    assert_eq!(run(cli_edges_json).unwrap(), ExitCode::SUCCESS);

    let mut cli_related = cli.clone();
    cli_related.command = Command::Related {
        files: vec![PathBuf::from("app/page.tsx")],
    };
    assert_eq!(run(cli_related.clone()).unwrap(), ExitCode::SUCCESS);

    let mut cli_related_json = cli_related;
    cli_related_json.json = true;
    assert_eq!(run(cli_related_json).unwrap(), ExitCode::SUCCESS);

    let mut cli_unique = cli.clone();
    cli_unique.assert_unique_test_ids = true;
    cli_unique.assert_unique_html_ids = true;
    assert_eq!(run(cli_unique).unwrap(), ExitCode::from(1));

    print_edges_text(&analysis.edges);
    let related = build_related_report(
        &root,
        &analysis.edges.edges,
        &[PathBuf::from("web/app/page.tsx")],
    );
    print_related_text(&related);
    let _ = serde_json::to_string_pretty(&analysis).unwrap();

    let mut cli_tests = cli.clone();
    cli_tests.command = Command::Tests {
        files: vec![PathBuf::from("web/app/page.tsx")],
    };
    assert_eq!(run(cli_tests.clone()).unwrap(), ExitCode::SUCCESS);

    let mut cli_tests_json = cli_tests;
    cli_tests_json.json = true;
    assert_eq!(run(cli_tests_json).unwrap(), ExitCode::SUCCESS);
}

#[test]
fn run_check_returns_success_for_fully_covered_project() {
    let root = fixture_path(&["nextjs-selectors", "selector-covered"]);
    let cli = Cli {
        root,
        config: None,
        playwright_config: vec![],
        project: None,
        json: true,
        assert_conditional_tests: false,
        allow_skipped_tests: false,
        assert_unique_test_ids: false,
        assert_unique_html_ids: false,
        command: Command::Check,
    };

    assert_eq!(run(cli).unwrap(), ExitCode::SUCCESS);
}

#[test]
fn run_check_fails_for_uncovered_selectors_without_uncovered_routes() {
    let root = fixture_path(&["nextjs-selectors", "selector-uncovered"]);
    let cli = Cli {
        root,
        config: None,
        playwright_config: vec![],
        project: None,
        json: true,
        assert_conditional_tests: false,
        allow_skipped_tests: false,
        assert_unique_test_ids: false,
        assert_unique_html_ids: false,
        command: Command::Check,
    };

    assert_eq!(run(cli).unwrap(), ExitCode::from(1));
}

#[test]
fn run_check_fails_for_duplicate_selectors_without_uncovered_coverage() {
    let root = fixture_path(&["nextjs-coverage", "sort-tiebreakers"]);
    let cli = Cli {
        root,
        config: None,
        playwright_config: vec![],
        project: None,
        json: true,
        assert_conditional_tests: false,
        allow_skipped_tests: false,
        assert_unique_test_ids: true,
        assert_unique_html_ids: false,
        command: Command::Check,
    };

    assert_eq!(run(cli).unwrap(), ExitCode::from(1));
}

#[test]
fn run_check_surfaces_settings_load_errors() {
    let root = fixture_path(&["nextjs-selectors", "selector-covered"]);
    let cli = Cli {
        root,
        config: Some(PathBuf::from("missing.no-mistakes.yml")),
        playwright_config: vec![],
        project: None,
        json: true,
        assert_conditional_tests: false,
        allow_skipped_tests: false,
        assert_unique_test_ids: false,
        assert_unique_html_ids: false,
        command: Command::Check,
    };

    let error = run(cli).unwrap_err();

    assert!(error.to_string().contains("config file does not exist"));
}

#[test]
fn analyze_surfaces_parser_errors() {
    let root = fixture_path(&["ast-snippets", "main", "invalid-test-source"]);
    let settings = Settings {
        frontend_root: "web/app".to_string(),
        playwright_configs: vec![],
        project: None,
        test_include: vec!["tests/**/*.spec.ts".to_string()],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![],
        navigation_helpers: vec![],
        selector_wrappers: vec![],
        selector_attributes: vec![],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec!["web/app".to_string()],
        selector_include: vec![],
        selector_exclude: vec![],
    };

    let err = analyze(&root, &settings).err().unwrap();
    assert!(err.to_string().contains("failed to parse"));

    let root = fixture_path(&["ast-snippets", "main", "invalid-selector-source"]);
    let settings = Settings {
        frontend_root: "web/app".to_string(),
        playwright_configs: vec![],
        project: None,
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![],
        navigation_helpers: vec![],
        selector_wrappers: vec![],
        selector_attributes: vec!["data-testid".to_string()],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec!["web/app".to_string()],
        selector_include: vec![],
        selector_exclude: vec![],
    };
    let selector_regexes = selectors::compile_selector_regexes(
        &settings.selector_attributes,
        &settings.component_selector_attributes,
    );
    let err = collect_app_selectors(&root, &settings, &selector_regexes)
        .err()
        .unwrap();
    assert!(err.to_string().contains("failed to parse"));
}

#[test]
fn analyze_test_file_with_selector_targets_extracts_edges() {
    // Uses a fixture that has data-testid selectors so app_selector_targets is
    // non-empty, exercising the `else` branch in analyze_test_file.
    let root = fixture_path(&["nextjs-selectors", "selector-covered"]);
    let settings = Settings {
        frontend_root: "web/app".to_string(),
        playwright_configs: vec![],
        project: None,
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![],
        navigation_helpers: vec![],
        selector_wrappers: vec![],
        selector_attributes: vec!["data-testid".to_string()],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec!["web/app".to_string()],
        selector_include: vec![],
        selector_exclude: vec![],
    };
    let analysis = analyze(&root, &settings).unwrap();
    let selector_edges: Vec<_> = analysis
        .edges
        .edges
        .iter()
        .filter(|e| matches!(e, crate::playwright::analysis::types::Edge::Selector { .. }))
        .collect();
    assert!(
        !selector_edges.is_empty(),
        "expected selector edges when app_selector_targets is non-empty"
    );
}

#[test]
fn analyze_test_occurrences_skips_non_test_route_inputs() {
    let root = fixture_path(&["nextjs-selectors", "selector-covered"]);
    let test_file = DiscoveredTestFile {
        path: root.join("tests/e2e/app.spec.ts"),
        contexts: vec![],
    };
    let route_index = context::RouteIndex {
        root: vec![context::RouteTarget {
            route_file: std::sync::Arc::new("web/app/page.tsx".to_string()),
            pattern: std::sync::Arc::new("/".to_string()),
            segments: Vec::new(),
        }],
        ..Default::default()
    };
    let selector_regexes =
        selectors::compile_selector_regexes(&["data-testid".to_string()], &BTreeMap::new());
    let selector_index = context::SelectorIndex::default();
    let context = TestAnalysisContext {
        root: &root,
        route_index: &route_index,
        selector_index: &selector_index,
        navigation_helpers: &[],
        selector_wrappers: &[],
        selector_regexes: &selector_regexes,
        test_policy: playwright_tests::TestPolicy {
            assert_conditional_tests: true,
            allow_skipped_tests: false,
        },
    };
    let occurrence = |value, status, scope| playwright_tests::TestOccurrence {
        value,
        status,
        scope,
        test_name: None,
        describe_path: Vec::new(),
        line: 1,
    };
    let raw_urls = vec![
        occurrence(
            "/".to_string(),
            playwright_tests::TestStatus::Conditional,
            playwright_tests::TestOccurrenceScope::Test,
        ),
        occurrence(
            "/".to_string(),
            playwright_tests::TestStatus::Active,
            playwright_tests::TestOccurrenceScope::TeardownHook,
        ),
        occurrence(
            "mailto:test@example.com".to_string(),
            playwright_tests::TestStatus::Active,
            playwright_tests::TestOccurrenceScope::Test,
        ),
    ];
    let playwright_selector = || {
        selectors::PlaywrightSelector::for_test(
            "data-testid",
            "save",
            selectors::SelectorMatcher::Exact("save".to_string()),
        )
    };
    let playwright_selectors = vec![
        playwright_tests::TestOccurrence {
            value: playwright_selector(),
            status: playwright_tests::TestStatus::Skipped,
            scope: playwright_tests::TestOccurrenceScope::Test,
            test_name: None,
            describe_path: Vec::new(),
            line: 1,
        },
        playwright_tests::TestOccurrence {
            value: playwright_selector(),
            status: playwright_tests::TestStatus::Active,
            scope: playwright_tests::TestOccurrenceScope::TeardownHook,
            test_name: None,
            describe_path: Vec::new(),
            line: 1,
        },
    ];

    let occurrences = crate::playwright::test_file_occurrences::TestFileOccurrences {
        variant: std::sync::Arc::new(
            crate::playwright::test_file_occurrences::VariantOccurrences {
                urls: raw_urls,
                selectors: playwright_selectors,
                helper_references: Vec::new(),
            },
        ),
        common: Default::default(),
    };
    let analysis = analyze_test_occurrences(&test_file, &context, &occurrences);

    assert!(analysis.edges.is_empty());
}
