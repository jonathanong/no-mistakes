use super::*;

#[test]
fn named_object_spread_keeps_source_member_imports() {
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
        &[NodeId::file(root.join("src/object-spread-named.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps.iter().any(|entry| {
        entry.node.as_file() == Some(root.join("src/object-spread-loaded.mts").as_path())
    }));
}

#[test]
fn object_literal_spread_call_keeps_source_member_imports() {
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
        &[NodeId::file(root.join("src/object-spread-call.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps.iter().any(|entry| {
        entry.node.as_file() == Some(root.join("src/object-spread-loaded.mts").as_path())
    }));
}

#[test]
fn later_object_spread_overwrite_drops_source_member_imports() {
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
        &[NodeId::file(root.join("src/object-spread-overwrite.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(!deps.iter().any(|entry| {
        entry.node.as_file() == Some(root.join("src/object-spread-loaded.mts").as_path())
    }));
}
