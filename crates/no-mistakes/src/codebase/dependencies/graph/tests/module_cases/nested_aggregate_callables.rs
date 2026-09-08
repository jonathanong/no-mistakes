use super::*;

#[test]
fn nested_aggregate_callable_members_reach_only_invoked_bodies_and_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/nested-callable-aggregates/fixture"),
    );
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let source = root.join("src/aggregate-callables.mts");
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .expect("aggregate callable graph must build");
    let deps = graph.deps_of(
        &[NodeId::file(source)],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let imports = deps
        .iter()
        .filter_map(|entry| entry.node.as_file())
        .collect::<HashSet<_>>();

    for path in [
        "object-called.mts",
        "field-called.mts",
        "field-reloaded.mts",
    ] {
        assert!(
            imports.contains(root.join("src").join(path).as_path()),
            "invoked aggregate member must retain {path}: {imports:?}"
        );
    }
    for path in ["object-unused.mts", "field-unused.mts"] {
        assert!(
            !imports.contains(root.join("src").join(path).as_path()),
            "uninvoked aggregate member must remain pruned: {path}"
        );
    }
}
