use super::*;

#[test]
fn imported_calls_follow_exported_namespace_member_aliases() {
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
    let consumer = root.join("src/namespace-alias-consumer.mts");
    let target = root.join("src/namespace-alias-target.mts");
    let calls = graph.deps_of(
        &[NodeId::file(consumer)],
        None,
        Some(&[EdgeKind::Call].into()),
    );

    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == target.as_path() && symbol.as_ref() == "run"
        ) && entry.via.contains(&EdgeKind::Call)
    }));
}
