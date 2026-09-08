use super::*;

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
    let calls = graph.deps_of(
        &[NodeId::file(&file)],
        None,
        Some(&[EdgeKind::Call].into()),
    );

    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { symbol, .. } if symbol.as_ref() == "run"
        )
    }));
    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file: target_file, symbol }
                if target_file.as_ref() == root.join("src/default-target.mts").as_path()
                    && symbol.as_ref() == "defaultTarget"
        )
    }));
    assert!(!calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file: target_file, symbol }
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
            NodeId::Symbol { file: target_file, symbol }
                if target_file.as_ref() == root.join("src/imported-target.mts").as_path()
                    && symbol.as_ref() == "importedTarget"
        )
    }));
    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file: target_file, symbol }
                if target_file.as_ref() == root.join("src/imported-target.mts").as_path()
                    && symbol.as_ref() == "reexportOnly"
        )
    }));
    assert!(calls.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::Symbol { file: target_file, symbol }
                if target_file.as_ref() == root.join("src/star-target.mts").as_path()
                    && symbol.as_ref() == "starTarget"
        )
    }));
}

#[test]
fn call_traces_have_stable_shortest_paths_and_respect_layers() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig { dir: root.clone(), paths: vec![], paths_dir: root.clone(), base_url: None };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan { calls: true, ..GraphBuildPlan::default() },
    )
    .unwrap();
    let file = root.join("src/call-edges.mts");
    let roots = graph.expand_call_roots(&[CallRoot::Function {
        file: file.clone(),
        symbol: "run".to_string(),
    }]);
    let module_roots = graph.expand_call_roots(&[CallRoot::Module(file.clone())]);
    assert_eq!(module_roots, vec![NodeId::file(&file)]);
    let vitest_roots = graph.expand_call_roots(&[CallRoot::Vitest { files: vec![file.clone()] }]);
    assert!(vitest_roots.contains(&NodeId::file(&file)));
    assert!(vitest_roots.iter().any(|node| node.display_name(&root).ends_with("#run")));
    assert!(vitest_roots.iter().any(|node| node.display_name(&root).ends_with("#target")));
    let direct = graph.call_traces(&roots, CallTraversal::Direct, None);
    assert!(direct.iter().any(|trace| trace.target.display_name(&root).ends_with("#target")));
    assert!(direct.iter().all(|trace| trace.nodes.len() == 2));
    let file_layer = graph.call_traces(&roots, CallTraversal::File, Some(4));
    assert!(file_layer.iter().all(|trace| trace.target.as_file() == Some(file.as_path())));
    assert!(file_layer.iter().all(|trace| trace.nodes.len() <= 3));
    assert_eq!(file_layer, graph.call_traces(&roots, CallTraversal::File, Some(4)));
}

#[test]
fn file_and_vitest_call_roots_include_global_only_callables() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig { dir: root.clone(), paths: vec![], paths_dir: root.clone(), base_url: None };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan { calls: true, ..GraphBuildPlan::default() },
    )
    .unwrap();
    let file = root.join("src/global-only.mts");
    let expected = NodeId::symbol(&file, "globalOnly");
    assert!(graph.expand_call_roots(&[CallRoot::File(file.clone())]).contains(&expected));
    assert!(graph
        .expand_call_roots(&[CallRoot::Vitest { files: vec![file] }])
        .contains(&expected));
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
    let file_facts = facts.get(&source).expect("fixture source must produce TS facts");
    assert!(file_facts
        .callable_aliases
        .iter()
        .any(|alias| alias.local == "invoke" && alias.target == "target"));
    assert!(file_facts.callable_scopes.iter().any(|scope| scope == "target"));
    assert!(file_facts
        .function_calls
        .iter()
        .any(|call| call.caller.is_none() && call.callee == "invoke"));
    assert!(reachable_function_scopes(file_facts).contains("target"));
    let deps = graph.deps_of(
        &[NodeId::file(source)],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps
        .iter()
        .any(|entry| entry.node.as_file() == Some(root.join("src/called.mts").as_path())));
}

#[test]
fn graph_includes_external_module_and_package_dependency_nodes() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-modules"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::all()).unwrap();

    let deps = graph.deps_of(&[NodeId::file(root.join("src/entry.mts"))], None, None);
    assert!(deps.iter().any(|entry| {
        entry.node == NodeId::module("@react/client")
            && entry.via.contains(&EdgeKind::Import)
    }));
    assert!(deps.iter().any(|entry| {
        entry.node == NodeId::file(root.join("packages/local/src/index.mts"))
            && entry.via.contains(&EdgeKind::WorkspaceImport)
    }));
    assert!(!deps
        .iter()
        .any(|entry| entry.node == NodeId::module("@local/pkg")));

    let manifest_deps = graph.deps_of(&[NodeId::file(root.join("package.json"))], None, None);
    assert!(manifest_deps.iter().any(|entry| {
        entry.node == NodeId::module("@react/server")
            && entry.via.contains(&EdgeKind::PackageDependency)
    }));
}

#[test]
fn node_builtin_imports_do_not_create_module_nodes() {
    let interner = PathInterner::new();
    assert_eq!(bare_module_node_in(&interner, "node:path"), None);
    assert_eq!(bare_module_node_in(&interner, "node:fs/promises"), None);
}

#[test]
fn import_fact_kinds_map_to_edge_kinds() {
    let mut import = ExtractedImport {
        specifier: "dep".to_string(),
        kind: ImportKind::Static,
        line: 1,
        function_scope: None,
        side_effect_only: false,
        re_export: false,
        runtime_reachable: false,
    };

    assert_eq!(edge_kind_for_import(&import), EdgeKind::Import);
    import.kind = ImportKind::Type;
    assert_eq!(edge_kind_for_import(&import), EdgeKind::TypeImport);
    import.kind = ImportKind::Dynamic;
    assert_eq!(edge_kind_for_import(&import), EdgeKind::DynamicImport);
    import.kind = ImportKind::Require;
    assert_eq!(edge_kind_for_import(&import), EdgeKind::Require);
    import.kind = ImportKind::RequireResolve;
    assert_eq!(edge_kind_for_import(&import), EdgeKind::RequireResolve);
}

#[test]
fn type_imports_in_exported_symbol_scopes_are_reachable() {
    let import = ExtractedImport {
        specifier: "./target.mts".to_string(),
        kind: ImportKind::Type,
        line: 1,
        function_scope: Some("PublicShape".to_string()),
        side_effect_only: false,
        re_export: false,
        runtime_reachable: false,
    };
    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        symbols: Some(std::sync::Arc::new(crate::codebase::ts_symbols::FileSymbols {
            exports: vec![crate::codebase::ts_symbols::Export {
                name: "PublicShape".to_string(),
                local: None,
                kind: crate::codebase::ts_symbols::ExportKind::TypeAlias,
                line: 1,
                is_type_only: true,
            }],
            imports: vec![],
        })),
        ..Default::default()
    };

    assert!(import_is_reachable(&import, &facts, &HashSet::new()));
}

#[test]
fn unknown_call_reachability_treats_none_as_conservative() {
    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        unknown_callers: vec![None],
        ..Default::default()
    };

    assert!(has_reachable_unknown_call(&facts, &HashSet::new()));
}

#[test]
fn unknown_call_reachability_treats_exported_callers_as_conservative() {
    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        exported_functions: vec!["exported".to_string()],
        unknown_callers: vec![Some("exported".to_string())],
        ..Default::default()
    };

    assert!(has_reachable_unknown_call(&facts, &HashSet::new()));
}

#[test]
fn pkg_name_scoped_no_subpath() {
    assert_eq!(package_name_from_spec("@x/api"), "@x/api");
}

#[test]
fn pkg_name_scoped_with_subpath() {
    assert_eq!(package_name_from_spec("@x/api/utils"), "@x/api");
}

#[test]
fn pkg_name_unscoped_no_subpath() {
    assert_eq!(package_name_from_spec("lodash"), "lodash");
}

#[test]
fn pkg_name_unscoped_with_subpath() {
    assert_eq!(package_name_from_spec("lodash/merge"), "lodash");
}
