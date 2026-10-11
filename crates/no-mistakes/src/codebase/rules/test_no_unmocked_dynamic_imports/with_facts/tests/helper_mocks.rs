use super::*;
use crate::codebase::dependencies::graph::GraphBuildPlan;

#[test]
fn imported_helper_mock_is_scoped_to_each_test_even_with_shared_dependency_cache() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-imported-helpers",
        ),
    );
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        files.clone(),
        crate::codebase::check_facts::CheckFactPlan {
            imports: true,
            dynamic_imports: true,
            source: true,
            ..Default::default()
        },
    );
    let visible = files.into_iter().collect::<crate::fx::PathSet>();
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig(None, &root).unwrap();
    let catalog = TsConfigCatalog::forced(&root, tsconfig.clone(), None);
    let resolver = ScopedImportResolver::new(&catalog, &visible);
    let graph_files = GraphFiles::from_files(shared.files().to_vec());
    let graph = DepGraph::build_with_plan_and_files(
        &root,
        &tsconfig,
        GraphBuildPlan::imports_and_workspace(),
        &graph_files,
    )
    .unwrap();
    let config = NoMistakesConfig::default();
    let cache = DashMap::new();

    for (test_name, target, should_report) in [
        ("tests/imported.test.mts", "src/covered-leaf.mts", false),
        ("tests/isolated.test.mts", "src/covered-leaf.mts", true),
        (
            "tests/mock-only-bridge.test.mts",
            "src/bridge-leaf.mts",
            true,
        ),
        (
            "tests/live-bridge-path.test.mts",
            "src/bridge-leaf.mts",
            false,
        ),
        (
            "tests/live-bridge-path.test.mts",
            "src/bridge-leaf.mts",
            false,
        ),
        (
            "tests/mock-only-bridge.test.mts",
            "src/bridge-leaf.mts",
            true,
        ),
        ("tests/isolated.test.mts", "src/covered-leaf.mts", true),
        ("tests/imported.test.mts", "src/covered-leaf.mts", false),
    ] {
        let result = per_test::analyze(
            per_test::Request {
                root: &root,
                config: &config,
                resolver: &resolver,
                graph: &graph,
                graph_files: &graph_files,
                visible_files: &visible,
                manual_mocks: &HashSet::new(),
                setup_mocks: &HashSet::new(),
                shared: &shared,
                dependency_cache: &cache,
                defer_suppression: false,
            },
            root.join(test_name),
        )
        .unwrap();
        let has_leaf_finding = result
            .reachable_findings
            .iter()
            .any(|entry| entry.finding.target.as_deref() == Some(target));
        assert_eq!(has_leaf_finding, should_report, "{test_name}: {target}");
    }
}
