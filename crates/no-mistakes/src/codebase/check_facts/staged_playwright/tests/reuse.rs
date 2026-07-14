use super::*;
use crate::codebase::dependencies::graph::{GraphBuildPlan, TsFactLookup};
use crate::codebase::ts_resolver::TsConfig;
use crate::codebase::ts_source::facts::TsFactPlan;
use crate::playwright::analysis::context::DiscoveredTestFile;
use crate::playwright::analysis::pipeline_occurrences::{
    prepare_test_files, CachedOccurrenceSelection, PrepareTestFilesOptions,
};
use crate::playwright::selectors::compile_selector_regexes;

#[test]
fn app_and_route_caches_are_isolated_by_exact_project_settings() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/playwright-cache-scopes"),
    );
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(&root);
    let mut project_a = occurrence_settings(&[], &["data-testid"]);
    project_a.project = Some("a".to_string());
    project_a.selector_roots = vec!["project-a".to_string()];
    project_a.frontend_root = "frontend-a/app".to_string();
    let mut project_b = project_a.clone();
    project_b.project = Some("b".to_string());
    project_b.selector_roots = vec!["project-b".to_string()];
    project_b.frontend_root = "frontend-b/app".to_string();

    let mut playwright = PlaywrightFactPlan::from_settings(
        &root,
        project_a.clone(),
        std::collections::HashMap::new(),
        false,
        &snapshot,
    )
    .unwrap();
    playwright.include(
        PlaywrightFactPlan::from_settings(
            &root,
            project_b.clone(),
            std::collections::HashMap::new(),
            false,
            &snapshot,
        )
        .unwrap(),
    );
    let source_files = vec![
        root.join("project-a/App.tsx"),
        root.join("project-b/App.tsx"),
    ];
    playwright.set_source_files(source_files.clone());
    let facts = crate::codebase::check_facts::collect_check_facts_with_graph_files_and_playwright(
        &root,
        Vec::new(),
        source_files,
        CheckFactPlan::default(),
        Some(playwright),
    );

    let selectors_a = facts
        .get_or_compute_app_selector_occurrences(&project_a, false, &|| {
            unreachable!("project A staged selector cache is ready")
        })
        .unwrap();
    let selectors_b = facts
        .get_or_compute_app_selector_occurrences(&project_b, false, &|| {
            unreachable!("project B staged selector cache is ready")
        })
        .unwrap();
    assert_eq!(selectors_a.len(), 1, "{selectors_a:?}");
    assert_eq!(selectors_b.len(), 1, "{selectors_b:?}");
    assert!(selectors_a[0].file.ends_with("project-a/App.tsx"));
    assert!(selectors_b[0].file.ends_with("project-b/App.tsx"));

    let routes_a = crate::playwright::analysis::pipeline_setup::collect_playwright_routes(
        &root,
        &project_a,
        false,
        true,
        Some(&facts),
        &snapshot,
    )
    .unwrap();
    let routes_b = crate::playwright::analysis::pipeline_setup::collect_playwright_routes(
        &root,
        &project_b,
        false,
        true,
        Some(&facts),
        &snapshot,
    )
    .unwrap();
    assert_eq!(routes_a[0].pattern, "/alpha");
    assert_eq!(routes_b[0].pattern, "/beta");
}

#[test]
fn merged_source_plans_preserve_physical_duplicate_selector_occurrences() {
    let root = root();
    let path = root.join("graph/duplicate.tsx");
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(&root);
    let settings = occurrence_settings(&[], &["data-a"]);
    let mut playwright = PlaywrightFactPlan::from_settings(
        &root,
        settings.clone(),
        std::collections::HashMap::new(),
        false,
        &snapshot,
    )
    .unwrap();
    playwright.set_source_files(vec![path.clone()]);
    playwright.set_app_source_files([path.clone()]);
    playwright.include(playwright.clone());

    let facts = collect_facts(
        &[],
        &["graph/duplicate.tsx"],
        CheckFactPlan::default(),
        playwright,
    );
    let occurrences = facts
        .get_or_compute_app_selector_occurrences(&settings, false, &|| {
            unreachable!("staged cache is ready")
        })
        .unwrap();

    assert_eq!(occurrences.len(), 2, "{occurrences:?}");
    assert!(occurrences.iter().all(|selector| selector.file == path));
}

#[test]
fn playwright_only_sources_limit_complete_graph_plan_metadata() {
    let source = root().join("graph/leaf.ts");
    let mut playwright = PlaywrightFactPlan::default();
    playwright.set_source_files(vec![source.clone()]);
    let facts = collect_facts(
        &[],
        &[],
        CheckFactPlan {
            symbols: true,
            graph: TsFactPlan::imports(),
            ..CheckFactPlan::default()
        },
        playwright,
    );

    assert!(facts.graph_file_universe_is_complete());
    assert_eq!(facts.graph_file_universe(), &[source]);
    assert_eq!(facts.graph_plan(), TsFactPlan::imports());
}

#[test]
fn explicit_imports_collect_each_partition_once_and_reuse_test_facts() {
    let graph = [
        "src/scoped.ts",
        "graph/entry.ts",
        "graph/leaf.ts",
        "graph/graph.spec.ts",
    ];
    let mut playwright = PlaywrightFactPlan::default();
    add_test(
        &mut playwright,
        "graph/graph.spec.ts",
        TestPolicy::default(),
    );
    add_test(
        &mut playwright,
        "tests/playwright-only.spec.ts",
        TestPolicy::default(),
    );
    let facts = collect_facts(
        &["src/scoped.ts"],
        &graph,
        CheckFactPlan {
            graph: TsFactPlan::imports(),
            ..CheckFactPlan::default()
        },
        playwright,
    );

    assert_eq!(facts.stats.files_parsed, graph.len() + 1);
    assert_eq!(facts.stats.files_discovered, graph.len() + 1);
    assert!(facts
        .get_playwright_facts(&root().join("graph/graph.spec.ts"))
        .is_some());
    assert!(facts
        .get_playwright_facts(&root().join("tests/playwright-only.spec.ts"))
        .is_some());
    let graph = crate::codebase::dependencies::graph::DepGraph::build_with_plan_file_list_config_and_check_facts(
        &root(),
        &TsConfig {
            dir: root(),
            paths_dir: root(),
            ..TsConfig::default()
        },
        GraphBuildPlan {
            route_imports: true,
            ..GraphBuildPlan::default()
        },
        paths(&graph),
        None,
        &facts,
    )
    .expect("complete staged facts avoid graph fallback failures");
    assert!(graph.contains_file(&root().join("graph/entry.ts")));
}

#[test]
fn cached_malformed_test_error_is_reused_or_skipped() {
    let path = root().join("tests/malformed.spec.ts");
    let mut playwright = PlaywrightFactPlan::default();
    add_test(
        &mut playwright,
        "tests/malformed.spec.ts",
        TestPolicy::default(),
    );
    let facts = collect_facts(&[], &[], CheckFactPlan::default(), playwright);
    let test_file = || DiscoveredTestFile {
        path: path.clone(),
        contexts: Vec::new(),
    };
    let regexes = compile_selector_regexes(&[], &BTreeMap::new());
    let settings = occurrence_settings(&[], &[]);

    let error = prepare_test_files(
        vec![test_file()],
        &settings,
        &regexes,
        PrepareTestFilesOptions {
            test_policy: TestPolicy::default(),
            skip_test_file_errors: false,
            facts: Some(&facts),
            selection: CachedOccurrenceSelection::Exact,
            module_resolution: None,
        },
    )
    .err()
    .expect("public analysis preserves cached parse error");
    assert!(error.to_string().contains("malformed.spec.ts"));
    let (prepared, _) = prepare_test_files(
        vec![test_file()],
        &settings,
        &regexes,
        PrepareTestFilesOptions {
            test_policy: TestPolicy::default(),
            skip_test_file_errors: true,
            facts: Some(&facts),
            selection: CachedOccurrenceSelection::Exact,
            module_resolution: None,
        },
    )
    .expect("optional graph analysis skips cached parse error");
    assert!(prepared.is_empty());
    let (standalone_skipped, _) = prepare_test_files(
        vec![test_file()],
        &settings,
        &regexes,
        PrepareTestFilesOptions {
            test_policy: TestPolicy::default(),
            skip_test_file_errors: true,
            facts: None,
            selection: CachedOccurrenceSelection::Exact,
            module_resolution: None,
        },
    )
    .expect("optional analysis skips a standalone parse error");
    assert!(standalone_skipped.is_empty());
    assert_eq!(facts.stats.files_parsed, 1);
    assert_eq!(facts.stats.parse_errors, 1);
}

#[test]
fn collection_stats_and_keys_are_deterministic() {
    let mut playwright = PlaywrightFactPlan::default();
    add_test(
        &mut playwright,
        "graph/graph.spec.ts",
        TestPolicy::default(),
    );
    let forward = collect_facts(
        &["src/scoped.ts", "graph/leaf.ts"],
        &["src/scoped.ts", "graph/entry.ts", "graph/graph.spec.ts"],
        CheckFactPlan::default(),
        playwright.clone(),
    );
    let reverse = collect_facts(
        &["graph/leaf.ts", "src/scoped.ts"],
        &["graph/graph.spec.ts", "graph/entry.ts", "src/scoped.ts"],
        CheckFactPlan::default(),
        playwright,
    );
    let keys = |facts: &CheckFactMap| {
        facts
            .ts
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
    };

    assert_eq!(
        forward.stats.files_discovered,
        reverse.stats.files_discovered
    );
    assert_eq!(forward.stats.files_parsed, reverse.stats.files_parsed);
    assert_eq!(forward.stats.parse_errors, reverse.stats.parse_errors);
    assert_eq!(keys(&forward), keys(&reverse));
}
