use super::*;

#[test]
fn sequence_callee_final_operands_produce_local_and_imported_call_edges() {
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
    let source = root.join("src/sequence-callees.mts");
    let imported = root.join("src/sequence-imported-target.mts");
    let calls = graph.deps_of(
        &[NodeId::file(source.clone())],
        None,
        Some(&[EdgeKind::Call].into()),
    );

    assert!(calls.iter().any(|entry| match &entry.node {
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == source.as_path() && symbol.as_ref() == "localTarget" =>
            {
                entry.via.contains(&EdgeKind::Call)
            }
            _ => false,
        }));
    assert!(calls.iter().any(|entry| match &entry.node {
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == imported.as_path() && symbol.as_ref() == "importedTarget" =>
            {
                entry.via.contains(&EdgeKind::Call)
            }
            _ => false,
        }));
    assert!(!calls.iter().any(|entry| {
        matches!(&entry.node, NodeId::Symbol { symbol, .. } if symbol.as_ref() == "dynamicFactory")
    }));

}
