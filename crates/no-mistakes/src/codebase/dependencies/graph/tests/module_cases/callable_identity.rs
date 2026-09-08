use super::*;

#[test]
fn block_local_immutable_alias_keeps_target_dynamic_import_reachable() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let source = root.join("src/block-alias-reachable.mts");
    let facts = collect_ts_facts(
        std::slice::from_ref(&source),
        TsFactPlan {
            function_calls: true,
            imports: true,
            ..TsFactPlan::default()
        },
    );
    let file_facts = facts
        .get(&source)
        .expect("fixture source must produce TS facts");
    let call = file_facts
        .function_calls
        .iter()
        .find(|call| call.callee == "invoke")
        .expect("block alias call must be collected");
    assert!(file_facts.callable_aliases.iter().any(|alias| {
        alias.local == "invoke"
            && alias.target == "target"
            && alias.binding_scope == call.callee_binding_scope.expect("block binding scope")
    }));

    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(source)],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    assert!(
        deps.iter()
            .any(|entry| { entry.node.as_file() == Some(root.join("src/called.mts").as_path()) })
    );
    assert!(
        !deps
            .iter()
            .any(|entry| { entry.node.as_file() == Some(root.join("src/uncalled.mts").as_path()) })
    );
}

#[test]
fn imported_callable_identity_comes_from_the_resolved_export() {
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
    let consumer = root.join("src/imported-callable-identity-consumer.mts");
    let target = root.join("src/imported-callable-identity-target.mts");
    let roots = graph.expand_call_roots(&[CallRoot::Function {
        file: consumer,
        symbol: "run".to_string(),
    }]);
    let is_target_symbol = |node: &NodeId, expected: &str| {
        matches!(
            node,
            NodeId::Symbol { file, symbol, .. }
                if file.as_ref() == target.as_path() && symbol.as_ref() == expected
        )
    };
    let direct = graph.call_traces(&roots, CallTraversal::Direct, None);
    assert!(
        direct
            .iter()
            .any(|trace| is_target_symbol(&trace.target, "actual"))
    );
    assert!(
        !direct
            .iter()
            .any(|trace| is_target_symbol(&trace.target, "aliasName")),
        "the importer alias must not select a same-spelled unrelated target binding",
    );
    assert!(
        graph
            .call_traces(&roots, CallTraversal::Transitive, None)
            .iter()
            .any(|trace| is_target_symbol(&trace.target, "actualLeaf")),
        "the resolved exported callable must retain its own downstream calls",
    );
}
