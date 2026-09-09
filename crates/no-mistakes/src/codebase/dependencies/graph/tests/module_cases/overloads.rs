use super::*;

#[test]
fn overload_implementation_has_one_function_root_and_reverse_call_edge() {
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
    let source = root.join("src/overloads.mts");
    let parse = graph.expand_call_roots(&[CallRoot::Function {
        file: source.clone(),
        symbol: "parse".to_string(),
    }]);
    let target = graph.expand_call_roots(&[CallRoot::Function {
        file: source.clone(),
        symbol: "target".to_string(),
    }]);
    assert_eq!(parse.len(), 1);
    assert_eq!(target.len(), 1);
    let direct = graph.call_traces(&parse, CallTraversal::Direct, None);

    assert!(direct.iter().any(|trace| trace.target == target[0]));
    assert!(
        graph
            .dependents_of_symbol_nodes(&target, None, Some(&[EdgeKind::Call].into()))
            .iter()
            .any(|entry| entry.node.as_file() == Some(source.as_path()))
    );
}
