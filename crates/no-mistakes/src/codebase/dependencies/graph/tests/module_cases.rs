use super::*;

#[test]
fn dynamic_imports_inside_uncalled_functions_are_pruned() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-call-narrowing"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
        .unwrap();
    let deps = graph.deps_of(
        &[NodeId::File(root.join("src/entry.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let paths: HashSet<_> = deps.iter().filter_map(|entry| entry.node.as_file()).collect();

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
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
        .unwrap();
    let deps = graph.deps_of(
        &[NodeId::File(root.join("src/unknown.mts"))],
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
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
        .unwrap();
    let deps = graph.deps_of(
        &[NodeId::File(root.join("src/export-named.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let paths: HashSet<_> = deps.iter().filter_map(|entry| entry.node.as_file()).collect();

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
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
        .unwrap();
    let deps = graph.deps_of(
        &[NodeId::File(root.join("src/export-default-identifier.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let paths: HashSet<_> = deps.iter().filter_map(|entry| entry.node.as_file()).collect();

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
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
        .unwrap();
    let deps = graph.deps_of(
        &[NodeId::File(root.join("src/export-nested.mts"))],
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
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
        .unwrap();
    let deps = graph.deps_of(
        &[NodeId::File(root.join("src/duplicate-name.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );
    let paths: HashSet<_> = deps.iter().filter_map(|entry| entry.node.as_file()).collect();

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
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
        .unwrap();
    let deps = graph.deps_of(
        &[NodeId::File(root.join("src/sibling.mts"))],
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
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
        .unwrap();
    let deps = graph.deps_of(
        &[NodeId::File(root.join("src/method.mts"))],
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
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
        .unwrap();
    let deps = graph.deps_of(
        &[NodeId::File(root.join("src/unknown-nested.mts"))],
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
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
        .unwrap();
    let deps = graph.deps_of(
        &[NodeId::File(root.join("src/unknown-uncalled.mts"))],
        None,
        Some(&[EdgeKind::DynamicImport].into()),
    );

    assert!(deps.is_empty());
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

    let deps = graph.deps_of(&[NodeId::File(root.join("src/entry.mts"))], None, None);
    assert!(deps.iter().any(|entry| {
        entry.node == NodeId::Module("@react/client".to_string())
            && entry.via.contains(&EdgeKind::Import)
    }));
    assert!(deps.iter().any(|entry| {
        entry.node == NodeId::File(root.join("packages/local/src/index.mts"))
            && entry.via.contains(&EdgeKind::WorkspaceImport)
    }));
    assert!(!deps
        .iter()
        .any(|entry| entry.node == NodeId::Module("@local/pkg".to_string())));

    let manifest_deps = graph.deps_of(&[NodeId::File(root.join("package.json"))], None, None);
    assert!(manifest_deps.iter().any(|entry| {
        entry.node == NodeId::Module("@react/server".to_string())
            && entry.via.contains(&EdgeKind::PackageDependency)
    }));
}

#[test]
fn node_builtin_imports_do_not_create_module_nodes() {
    assert_eq!(bare_module_node("node:path"), None);
    assert_eq!(bare_module_node("node:fs/promises"), None);
}

#[test]
fn import_fact_kinds_map_to_edge_kinds() {
    let mut import = ExtractedImport {
        specifier: "dep".to_string(),
        kind: ImportKind::Static,
        function_scope: None,
    };

    assert_eq!(edge_kind_for_import(&import), EdgeKind::Import);
    import.kind = ImportKind::Type;
    assert_eq!(edge_kind_for_import(&import), EdgeKind::TypeImport);
    import.kind = ImportKind::Dynamic;
    assert_eq!(edge_kind_for_import(&import), EdgeKind::DynamicImport);
    import.kind = ImportKind::Require;
    assert_eq!(edge_kind_for_import(&import), EdgeKind::Require);
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
