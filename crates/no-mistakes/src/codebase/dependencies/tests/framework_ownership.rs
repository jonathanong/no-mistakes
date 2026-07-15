use super::*;

fn materialized_fixture() -> tempfile::TempDir {
    let source = fixture_root("graph-split-route-http-config");
    crate::test_support::materialize_saved_fixture(&source)
}

fn prepare(root: &Path, plan: graph::GraphBuildPlan) -> anyhow::Result<SharedTraversalContext> {
    SharedTraversalContext::prepare_with_framework_plan(
        root.to_path_buf(),
        None,
        None,
        plan,
        crate::codebase::test_discovery::FrameworkPreparationPlan::for_graph(plan),
    )
}

#[test]
fn vitest_fallback_excludes_config_only_playwright_owned_paths() {
    let fixture = materialized_fixture();
    let root = fixture.path().canonicalize().unwrap();
    let framework_plan = crate::codebase::test_discovery::FrameworkPreparationPlan::for_runners([
        crate::codebase::test_discovery::TestRunner::Vitest,
    ]);

    crate::ast::begin_parse_count(&root);
    let shared = SharedTraversalContext::prepare_with_framework_plan(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan::default(),
        framework_plan,
    )
    .unwrap();
    let prepared = shared
        .prepared_test_projects
        .as_ref()
        .expect("Vitest project discovery is prepared");
    let discovered = crate::codebase::test_discovery::discover_tests_from_prepared_projects(
        &root,
        shared.config(),
        crate::codebase::test_discovery::TestRunner::Vitest,
        prepared,
        shared.visible_paths().paths_for(&root).as_ref(),
        shared.tsconfig(),
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    for owned in ["routes/custom-owned.test.mts", "http/custom-owned.test.mts"] {
        assert!(!discovered.tests.contains(&root.join(owned)), "{owned}");
    }
    assert_eq!(
        counts.get(&root.join("custom-playwright.config.mts")),
        Some(&1),
        "{counts:#?}"
    );
}

fn assert_owned_route_is_filtered(
    plan: graph::GraphBuildPlan,
    kind: graph::EdgeKind,
    included: &str,
    excluded: &str,
) {
    let fixture = materialized_fixture();
    let root = fixture.path().canonicalize().unwrap();
    let client = root.join("src/client.ts");

    crate::ast::begin_parse_count(&root);
    let mut shared = prepare(&root, plan).unwrap();
    let graph = shared.canonical_graph().unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let edges = graph
        .dependencies_of_node(&graph::NodeId::File(client))
        .cloned()
        .unwrap_or_default();

    assert!(edges.iter().any(|(node, edge_kind)| {
        *edge_kind == kind && node.as_file() == Some(root.join(included).as_path())
    }));
    assert!(edges.iter().all(|(node, edge_kind)| {
        *edge_kind != kind || node.as_file() != Some(root.join(excluded).as_path())
    }));
    assert_eq!(
        counts.get(&root.join("custom-playwright.config.mts")),
        Some(&1),
        "{counts:#?}"
    );
}

#[test]
fn route_only_graph_excludes_config_only_playwright_owned_routes() {
    assert_owned_route_is_filtered(
        graph::GraphBuildPlan {
            routes: true,
            ..Default::default()
        },
        graph::EdgeKind::RouteRef,
        "routes/users.mts",
        "routes/custom-owned.test.mts",
    );
}

#[test]
fn http_only_graph_excludes_config_only_playwright_owned_routes() {
    assert_owned_route_is_filtered(
        graph::GraphBuildPlan {
            http: true,
            ..Default::default()
        },
        graph::EdgeKind::HttpCall,
        "http/users.mts",
        "http/custom-owned.test.mts",
    );
}
