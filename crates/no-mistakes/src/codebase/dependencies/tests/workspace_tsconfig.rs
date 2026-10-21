use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/workspace-resolution"),
    )
}

#[test]
fn shared_traversal_prepares_workspace_config_ownership() {
    let root = fixture_root();
    let allowed =
        std::collections::HashSet::from([crate::codebase::dependencies::EdgeKind::Import]);
    let plan = crate::codebase::dependencies::graph::GraphBuildPlan::from_allowed(Some(&allowed));
    let framework_plan = crate::codebase::test_discovery::FrameworkPreparationPlan::for_graph(plan);
    let mut shared =
        crate::codebase::dependencies::SharedTraversalContext::prepare_with_framework_plan(
            root.clone(),
            None,
            None,
            plan,
            framework_plan,
        )
        .unwrap();
    assert_eq!(
        shared
            .tsconfig_catalog
            .provenance_for(&root.join("apps/web/src/entry.ts"))
            .config
            .as_deref(),
        Some(root.join("apps/web/tsconfig.json").as_path())
    );
    let resolver = crate::codebase::ts_resolver::ScopedImportResolver::from_lookup(
        &shared.tsconfig_catalog,
        &shared.graph_files,
        None,
    );
    assert_eq!(
        resolver.resolve("@runtime/value", &root.join("apps/web/src/entry.ts")),
        Some(root.join("apps/web/src/runtime/value.ts"))
    );
    let graph = shared.request_graph(plan).unwrap();
    let entry = crate::codebase::dependencies::NodeId::file(root.join("apps/web/src/entry.ts"));
    let dependencies = graph.dependencies_of_node(&entry).unwrap();
    assert!(dependencies.iter().any(|(node, _)| {
        node == &crate::codebase::dependencies::NodeId::file(
            root.join("apps/web/src/runtime/value.ts"),
        )
    }));
}
