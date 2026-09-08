use super::*;

#[test]
fn static_class_members_resolve_without_guessing_function_object_members() {
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
    let source = root.join("src/static-class-members.mts");
    let calls = graph.deps_of(
        &[NodeId::file(source.clone())],
        None,
        Some(&[EdgeKind::Call].into()),
    );

    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == source.as_path() && symbol.as_ref() == "Service/run"
        ) && entry.via.contains(&EdgeKind::Call)
    }));
    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == source.as_path() && symbol.as_ref() == "ExpressionService/run"
        ) && entry.via.contains(&EdgeKind::Call)
    }));
    assert!(!calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == source.as_path() && symbol.as_ref() == "ExpressionService/instance"
        ) && entry.via.contains(&EdgeKind::Call)
    }));
    assert!(!calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == source.as_path() && symbol.as_ref() == "api/run"
        ) && entry.via.contains(&EdgeKind::Call)
    }));
}
