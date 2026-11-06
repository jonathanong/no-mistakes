use super::*;

#[test]
fn module_initialization_excludes_main_guards_but_preserves_direct_calls() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-import-main"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            imports: true,
            ..Default::default()
        },
    )
    .unwrap();
    let file = root.join("main.ts");
    let roots = graph.expand_call_roots(&[CallRoot::Module(file.clone())]);
    let sources = graph.call_source_nodes(&roots, CallTraversal::Transitive, None);
    for name in ["plain", "strict", "conjunction", "command", "namedCallback"] {
        assert!(
            !sources.iter().any(
                |node| matches!(node, NodeId::Symbol { symbol, .. } if symbol.as_ref() == name)
            ),
            "{name}"
        );
    }
    for name in ["alternate", "negated", "after"] {
        assert!(
            sources.iter().any(
                |node| matches!(node, NodeId::Symbol { symbol, .. } if symbol.as_ref() == name)
            ),
            "{name}"
        );
    }
    let callback = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.source_callee == "callbackEffect")
        .unwrap();
    let callback_owner = NodeId::callable(
        &file,
        callback.caller.as_deref().unwrap(),
        callback.caller_id.unwrap(),
    );
    assert!(!sources.contains(&callback_owner));
    assert!(!graph
        .resolved_call_sites()
        .iter()
        .any(|site| site.file == file
            && site.caller_id.is_none()
            && ["plain", "strict", "conjunction", "command", "namedCallback"]
                .contains(&site.source_callee.as_str())));
    let direct = graph.expand_call_roots(&[CallRoot::Module(root.join("direct.test.ts"))]);
    let direct_sources = graph.call_source_nodes(&direct, CallTraversal::Transitive, None);
    assert!(direct_sources
        .iter()
        .any(|node| matches!(node, NodeId::Symbol { symbol, .. } if symbol.as_ref() == "command")));
    assert!(graph
        .resolved_call_sites()
        .iter()
        .any(|site| site.source_callee == "commandEffect"));
}
