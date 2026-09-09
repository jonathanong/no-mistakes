use super::*;

#[test]
fn object_setter_writes_keep_setter_imports_reachable() {
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
        &[NodeId::file(root.join("src/object-setter-write.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps.iter().any(|entry| {
        entry.node.as_file() == Some(root.join("src/object-setter-loaded.mts").as_path())
    }));
}

#[test]
fn object_setter_updates_keep_setter_imports_reachable() {
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
        &[NodeId::file(root.join("src/object-setter-update.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps.iter().any(|entry| {
        entry.node.as_file() == Some(root.join("src/object-setter-loaded.mts").as_path())
    }));
}

#[test]
fn object_data_property_writes_drop_replaced_member_imports() {
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
        &[NodeId::file(root.join("src/object-setter-data-write.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(!deps.iter().any(|entry| {
        entry.node.as_file() == Some(root.join("src/object-setter-data-loaded.mts").as_path())
    }));
}
