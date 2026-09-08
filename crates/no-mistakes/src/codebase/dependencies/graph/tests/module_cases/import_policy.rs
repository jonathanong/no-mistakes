use super::*;

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
        entry.node == NodeId::module("@react/client") && entry.via.contains(&EdgeKind::Import)
    }));
    assert!(deps.iter().any(|entry| {
        entry.node == NodeId::file(root.join("packages/local/src/index.mts"))
            && entry.via.contains(&EdgeKind::WorkspaceImport)
    }));
    assert!(
        !deps
            .iter()
            .any(|entry| entry.node == NodeId::module("@local/pkg"))
    );

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
        function_scope_id: None,
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
        function_scope_id: None,
        side_effect_only: false,
        re_export: false,
        runtime_reachable: false,
    };
    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        symbols: Some(std::sync::Arc::new(
            crate::codebase::ts_symbols::FileSymbols {
                exports: vec![crate::codebase::ts_symbols::Export {
                    name: "PublicShape".to_string(),
                    local: None,
                    kind: crate::codebase::ts_symbols::ExportKind::TypeAlias,
                    line: 1,
                    is_type_only: true,
                }],
                imports: vec![],
            },
        )),
        ..Default::default()
    };

    assert!(import_is_reachable(&import, &facts, &HashSet::new()));
}

#[test]
fn unknown_call_reachability_treats_none_as_conservative() {
    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        unknown_calls: vec![crate::codebase::dependencies::extract::UnknownCall {
            caller: None,
            caller_id: None,
            line: 1,
            offset: 0,
            invocation: crate::codebase::dependencies::extract::InvocationKind::Call,
        }],
        ..Default::default()
    };

    assert!(has_reachable_unknown_call(&facts, &HashSet::new()));
}

#[test]
fn unknown_call_reachability_treats_exported_callers_as_conservative() {
    let facts = crate::codebase::ts_source::facts::TsFileFacts {
        exported_functions: vec!["exported".to_string()],
        unknown_calls: vec![crate::codebase::dependencies::extract::UnknownCall {
            caller: Some("exported".to_string()),
            caller_id: None,
            line: 1,
            offset: 0,
            invocation: crate::codebase::dependencies::extract::InvocationKind::Call,
        }],
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
