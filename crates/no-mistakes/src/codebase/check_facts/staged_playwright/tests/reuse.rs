use super::*;
use crate::codebase::dependencies::graph::{GraphBuildPlan, TsFactLookup};
use crate::codebase::ts_resolver::TsConfig;
use crate::codebase::ts_source::facts::TsFactPlan;
use crate::playwright::analysis::context::DiscoveredTestFile;
use crate::playwright::analysis::pipeline_occurrences::{
    prepare_test_files, CachedOccurrenceSelection,
};
use crate::playwright::selectors::compile_selector_regexes;

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
        TestPolicy::default(),
        false,
        Some(&facts),
        CachedOccurrenceSelection::Exact,
    )
    .err()
    .expect("public analysis preserves cached parse error");
    assert!(error.to_string().contains("malformed.spec.ts"));
    let (prepared, _) = prepare_test_files(
        vec![test_file()],
        &settings,
        &regexes,
        TestPolicy::default(),
        true,
        Some(&facts),
        CachedOccurrenceSelection::Exact,
    )
    .expect("optional graph analysis skips cached parse error");
    assert!(prepared.is_empty());
    let (standalone_skipped, _) = prepare_test_files(
        vec![test_file()],
        &settings,
        &regexes,
        TestPolicy::default(),
        true,
        None,
        CachedOccurrenceSelection::Exact,
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
