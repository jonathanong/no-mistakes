use super::*;

fn imported_class_graph() -> (PathBuf, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    (root, graph)
}

#[test]
fn named_imported_class_static_members_resolve() {
    let (root, graph) = imported_class_graph();
    let consumer = root.join("src/imported-class-named.mts");
    let target = root.join("src/imported-class-service.mts");
    let calls = graph.deps_of(
        &[NodeId::file(consumer)],
        None,
        Some(&[EdgeKind::Call].into()),
    );

    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == target.as_path() && symbol.as_ref() == "Service/run"
        ) && entry.via.contains(&EdgeKind::Call)
    }));
    assert!(!calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == target.as_path() && symbol.as_ref() == "Service"
        ) && entry.via.contains(&EdgeKind::Call)
    }));
}

#[test]
fn default_imported_class_static_members_resolve() {
    let (root, graph) = imported_class_graph();
    let consumer = root.join("src/imported-class-default.mts");
    let target = root.join("src/imported-class-default-service.mts");
    let calls = graph.deps_of(
        &[NodeId::file(consumer)],
        None,
        Some(&[EdgeKind::Call].into()),
    );

    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == target.as_path() && symbol.as_ref() == "Service/run"
        ) && entry.via.contains(&EdgeKind::Call)
    }));
    assert!(!calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == target.as_path() && symbol.as_ref() == "Service"
        ) && entry.via.contains(&EdgeKind::Call)
    }));
}

#[test]
fn namespace_imported_class_members_stay_on_the_export_path() {
    let (root, graph) = imported_class_graph();
    let consumer = root.join("src/imported-class-namespace.mts");
    let target = root.join("src/imported-class-service.mts");
    let calls = graph.deps_of(
        &[NodeId::file(consumer)],
        None,
        Some(&[EdgeKind::Call].into()),
    );

    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == target.as_path() && symbol.as_ref() == "Service"
        ) && entry.via.contains(&EdgeKind::Call)
    }));
    assert!(!calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == target.as_path() && symbol.as_ref() == "Service/run"
        ) && entry.via.contains(&EdgeKind::Call)
    }));
}
