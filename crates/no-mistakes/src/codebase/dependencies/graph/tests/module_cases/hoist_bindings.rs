use super::*;

#[test]
fn hoisted_helper_inside_arrow_body_keeps_dynamic_import_reachable() {
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
        &[NodeId::file(root.join("src/arrow-hoisted-helper.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps.iter().any(|entry| {
        entry.node.as_file() == Some(root.join("src/called.mts").as_path())
    }));
}
