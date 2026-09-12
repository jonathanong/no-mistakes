use super::*;

mod alias_tdz;
mod accessor_kind;
mod constructor_reachability;
mod callable_identity;
mod duplicate_export_identity;
mod call_apply_reachability;
mod callback_and_getter_reachability;
mod hoist_bindings;
mod import_policy;
mod imported_class_statics;
mod namespace_alias;
mod namespace_external;
mod nested_aggregate_callables;
mod object_setter_reachability;
mod object_spread_reachability;
mod overloads;
mod sequence_callees;
mod static_class_members;
mod this_member_reachability;

#[test]
fn call_edges_are_opt_in_and_follow_lexical_function_scopes() {
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
    let file = root.join("src/call-edges.mts");
    let calls = graph.deps_of(&[NodeId::file(&file)], None, Some(&[EdgeKind::Call].into()));

    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { symbol, .. } if symbol.as_ref() == "run"
        )
    }));
    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file: target_file, symbol, .. }
                if target_file.as_ref() == root.join("src/default-target.mts").as_path()
                    && symbol.as_ref() == "defaultTarget"
        )
    }));
    assert!(!calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file: target_file, symbol, .. }
                if target_file.as_ref() == root.join("src/imported-target.mts").as_path()
                    && symbol.as_ref() == "importedTarget/member"
        )
    }));
    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { symbol, .. } if symbol.as_ref() == "target"
        )
    }));
    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file: target_file, symbol, .. }
                if target_file.as_ref() == root.join("src/imported-target.mts").as_path()
                    && symbol.as_ref() == "importedTarget"
        )
    }));
    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file: target_file, symbol, .. }
                if target_file.as_ref() == root.join("src/imported-target.mts").as_path()
                    && symbol.as_ref() == "reexportOnly"
        )
    }));
    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file: target_file, symbol, .. }
                if target_file.as_ref() == root.join("src/star-target.mts").as_path()
                    && symbol.as_ref() == "starTarget"
        )
    }));
}

#[test]
fn call_traces_have_stable_shortest_paths_and_respect_layers() {
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
    let file = root.join("src/call-edges.mts");
    let roots = graph.expand_call_roots(&[CallRoot::Function {
        file: file.clone(),
        symbol: "run".to_string(),
    }]);
    let module_roots = graph.expand_call_roots(&[CallRoot::Module(file.clone())]);
    assert_eq!(module_roots, vec![NodeId::file(&file)]);
    let collection_roots = graph.expand_call_roots(&[CallRoot::Files {
        files: vec![file.clone()],
    }]);
    assert!(collection_roots.contains(&NodeId::file(&file)));
    assert!(collection_roots
        .iter()
        .any(|node| node.display_name(&root).ends_with("#run")));
    assert!(collection_roots
        .iter()
        .any(|node| node.display_name(&root).ends_with("#target")));
    let direct = graph.call_traces(&roots, CallTraversal::Direct, None);
    assert!(direct
        .iter()
        .any(|trace| trace.target.display_name(&root).ends_with("#target")));
    assert!(direct.iter().all(|trace| trace.nodes.len() == 2));
    let file_layer = graph.call_traces(&roots, CallTraversal::File, Some(4));
    assert!(file_layer
        .iter()
        .all(|trace| trace.target.as_file() == Some(file.as_path())));
    assert!(file_layer.iter().all(|trace| trace.nodes.len() <= 3));
    assert_eq!(
        file_layer,
        graph.call_traces(&roots, CallTraversal::File, Some(4))
    );
}

#[test]
fn file_and_collection_call_roots_include_global_only_callables() {
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
    let file = root.join("src/global-only.mts");
    assert!(graph
        .expand_call_roots(&[CallRoot::File(file.clone())])
        .iter()
        .any(|node| matches!(node, NodeId::Symbol { file: owner, symbol, .. } if owner.as_ref() == file.as_path() && symbol.as_ref() == "globalOnly")));
    assert!(graph
        .expand_call_roots(&[CallRoot::Files { files: vec![file] }])
        .iter()
        .any(|node| matches!(node, NodeId::Symbol { symbol, .. } if symbol.as_ref() == "globalOnly")));
}

#[test]
fn dynamic_imports_inside_uncalled_functions_are_pruned() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/entry.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let paths: HashSet<_> = deps
        .iter()
        .filter_map(|entry| entry.node.as_file())
        .collect();

    assert!(paths.contains(root.join("src/called.mts").as_path()));
    assert!(!paths.contains(root.join("src/uncalled.mts").as_path()));
}

#[test]
fn unknown_top_level_calls_keep_function_scoped_dynamic_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/unknown.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps
        .iter()
        .any(|entry| entry.node.as_file() == Some(root.join("src/uncalled.mts").as_path())));
}

#[test]
fn named_reexports_keep_function_scoped_dynamic_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/export-named.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let paths: HashSet<_> = deps
        .iter()
        .filter_map(|entry| entry.node.as_file())
        .collect();

    assert!(paths.contains(root.join("src/called.mts").as_path()));
    assert!(!paths.contains(root.join("src/uncalled.mts").as_path()));
}

#[test]
fn default_identifier_exports_keep_function_scoped_dynamic_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/export-default-identifier.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let paths: HashSet<_> = deps
        .iter()
        .filter_map(|entry| entry.node.as_file())
        .collect();

    assert!(paths.contains(root.join("src/called.mts").as_path()));
    assert!(!paths.contains(root.join("src/uncalled.mts").as_path()));
}

#[test]
fn nested_functions_inside_exported_functions_are_not_exported() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/export-nested.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps.is_empty());
}

#[test]
fn same_named_nested_functions_do_not_share_reachability() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/duplicate-name.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let paths: HashSet<_> = deps
        .iter()
        .filter_map(|entry| entry.node.as_file())
        .collect();

    assert!(paths.contains(root.join("src/called.mts").as_path()));
    assert!(!paths.contains(root.join("src/uncalled.mts").as_path()));
}

#[test]
fn nested_function_calls_resolve_sibling_scopes() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/sibling.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps
        .iter()
        .any(|entry| entry.node.as_file() == Some(root.join("src/called.mts").as_path())));
}

#[test]
fn same_named_sibling_bindings_reach_only_the_invoked_declaration() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/same-name-siblings.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let paths = deps
        .iter()
        .filter_map(|entry| entry.node.as_file())
        .collect::<HashSet<_>>();

    assert!(paths.contains(root.join("src/sibling-first.mts").as_path()));
    assert!(!paths.contains(root.join("src/sibling-second.mts").as_path()));
}

#[test]
fn nested_callers_resolve_aliases_declared_by_lexical_parents() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/nested-parent-alias.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps.iter().any(|entry| {
        entry.node.as_file() == Some(root.join("src/nested-parent-alias-target.mts").as_path())
    }));

    let call_graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..GraphBuildPlan::default()
        },
    )
    .unwrap();
    assert!(call_graph.resolved_call_sites().iter().any(|site| {
        site.file == root.join("src/nested-parent-alias.mts")
            && site.source_callee == "load"
            && matches!(
                &site.target,
                ResolvedCallTarget::RepositoryFunction { file, scope }
                    if file == &root.join("src/nested-parent-alias.mts") && scope == "target"
            )
    }));
}

#[test]
fn uncalled_method_dynamic_imports_are_pruned() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/method.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps.is_empty());
}

#[test]
fn unknown_calls_inside_reachable_functions_keep_function_scoped_dynamic_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/unknown-nested.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps
        .iter()
        .any(|entry| entry.node.as_file() == Some(root.join("src/uncalled.mts").as_path())));
}

#[test]
fn unknown_calls_inside_unreachable_functions_do_not_broaden_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let deps = graph.deps_of(
        &[NodeId::file(root.join("src/unknown-uncalled.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps.is_empty());
}

#[test]
fn immutable_local_alias_calls_keep_target_function_imports_reachable() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let source = root.join("src/alias-reachable.mts");
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
    assert!(file_facts
        .callable_aliases
        .iter()
        .any(|alias| alias.local == "invoke" && alias.target == "target"));
    assert!(file_facts
        .callable_scopes
        .iter()
        .any(|scope| scope == "target"));
    assert!(file_facts
        .function_calls
        .iter()
        .any(|call| call.caller.is_none() && call.callee == "invoke"));
    let reachable = reachable_function_scopes(file_facts);
    assert!(file_facts
        .callable_scope_ids
        .iter()
        .any(|(id, scope)| scope == "target" && reachable.contains(id)));
    let deps = graph.deps_of(
        &[NodeId::file(source)],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps
        .iter()
        .any(|entry| entry.node.as_file() == Some(root.join("src/called.mts").as_path())));
}
