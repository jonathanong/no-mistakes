use super::*;

#[test]
fn constructor_imports_require_construction_and_prune_other_members() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/constructor-reachability.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let paths: HashSet<_> = deps
        .iter()
        .filter_map(|entry| entry.node.as_file())
        .collect();

    assert!(paths.contains(root.join("src/constructor-called.mts").as_path()));
    assert!(paths.contains(root.join("src/instance-field-called.mts").as_path()));
    assert!(paths.contains(root.join("src/static-field-eager.mts").as_path()));
    assert!(!paths.contains(root.join("src/constructor-unused-method.mts").as_path()));
    assert!(!paths.contains(root.join("src/constructor-plain-call.mts").as_path()));
    assert!(!paths.contains(root.join("src/instance-field-unused.mts").as_path()));
}
