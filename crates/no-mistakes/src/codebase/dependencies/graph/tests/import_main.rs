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
    for name in ["alternate", "negated", "after", "shared"] {
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
    for root in [
        CallRoot::File(file.clone()),
        CallRoot::Files {
            files: vec![file.clone()],
        },
    ] {
        let all_sources = graph.expand_call_roots(&[root]);
        for name in ["plain", "strict", "conjunction", "command", "namedCallback"] {
            let site = graph
                .resolved_call_sites()
                .iter()
                .find(|site| site.file == file && site.source_callee == name)
                .unwrap();
            assert_eq!(site.caller.as_deref(), Some("<entry-point>"));
            let owner = NodeId::callable(
                &file,
                site.caller.as_deref().unwrap(),
                site.caller_id.unwrap(),
            );
            assert!(all_sources.contains(&owner), "{name}");
            assert!(!sources.contains(&owner), "{name}");
        }
        assert!(all_sources.contains(&callback_owner));
    }
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
