use super::*;

#[test]
fn alias_and_direct_calls_before_declaration_do_not_resolve() {
    let (root, graph) = alias_tdz_graph();
    let file = root.join("src/alias-tdz-ordering.mts");
    for symbol_name in ["beforeAlias", "directBefore"] {
        let targets = graph.call_traces(
            &[symbol(&file, symbol_name)],
            CallTraversal::Direct,
            None,
        );
        assert!(
            targets.is_empty(),
            "{symbol_name} must not resolve a TDZ call"
        );
    }
}

#[test]
fn alias_call_after_declaration_resolves_to_the_target() {
    let (root, graph) = alias_tdz_graph();
    let file = root.join("src/alias-tdz-ordering.mts");
    let targets = graph.call_traces(
        &[symbol(&file, "afterAlias")],
        CallTraversal::Direct,
        None,
    );

    assert_eq!(targets.len(), 1);
    assert!(has_symbol(&targets[0].target, &file, "target"));
}

#[test]
fn deferred_closure_may_alias_a_later_direct_binding_after_initialization() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let source = root.join("src/alias-tdz-ordering.mts");
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
    assert!(
        file_facts.callable_aliases.iter().any(|alias| {
            alias.local == "inner"
                && alias.target == "later"
                && alias.declared_at > 0
        }),
        "nested const inner = later must be materialized after later exists"
    );

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
            .any(|entry| { entry.node.as_file() == Some(root.join("src/uncalled.mts").as_path()) }),
        "before-only and early-plus-late nested calls must not resolve later"
    );
}

fn alias_tdz_graph() -> (PathBuf, DepGraph) {
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
            imports: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    (root, graph)
}

fn symbol(path: &Path, name: &str) -> NodeId {
    NodeId::symbol(path, name)
}

fn has_symbol(node: &NodeId, path: &Path, name: &str) -> bool {
    matches!(
        node,
        NodeId::Symbol { file, symbol, .. }
            if file.as_ref() == path && symbol.as_ref() == name
    )
}
