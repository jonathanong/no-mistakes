use super::*;

fn call_graph() -> (PathBuf, DepGraph) {
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
fn namespace_reexport_from_external_module_does_not_invent_a_file_target() {
    let (root, graph) = call_graph();
    let consumer = root.join("src/namespace-external-consumer.mts");
    let calls = graph.deps_of(
        &[NodeId::file(consumer)],
        None,
        Some(&[EdgeKind::Call].into()),
    );
    assert!(
        !calls.iter().any(|entry| entry.node.as_file().is_some()),
        "external namespace reexports must not resolve to a repository file: {calls:#?}"
    );
}

#[test]
fn namespace_reexport_from_missing_relative_module_does_not_invent_a_file_target() {
    let (root, graph) = call_graph();
    let consumer = root.join("src/namespace-missing-consumer.mts");
    let calls = graph.deps_of(
        &[NodeId::file(consumer)],
        None,
        Some(&[EdgeKind::Call].into()),
    );
    assert!(
        !calls.iter().any(|entry| entry.node.as_file().is_some()),
        "missing relative namespace reexports must stay unresolved: {calls:#?}"
    );
}

#[test]
fn exported_namespace_member_aliases_to_external_or_missing_modules_stay_unresolved() {
    let (root, graph) = call_graph();
    for consumer in [
        "src/namespace-external-alias-consumer.mts",
        "src/namespace-missing-alias-consumer.mts",
    ] {
        let calls = graph.deps_of(
            &[NodeId::file(root.join(consumer))],
            None,
            Some(&[EdgeKind::Call].into()),
        );
        assert!(
            !calls.iter().any(|entry| entry.node.as_file().is_some()),
            "{consumer} must not resolve to a repository file: {calls:#?}"
        );
    }
}
