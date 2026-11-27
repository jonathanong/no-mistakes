#[test]
fn low_level_collectors_cover_empty_invalid_and_non_visible_branches() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("codebase-intel"));
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    let package = root.join("package.json");
    let web_entry = root.join("packages/web/src/index.tsx");
    let hidden = root.join("packages/api/src/index.mts");
    let graph_files = GraphFiles::from_parts(
        vec![package.clone(), web_entry.clone(), hidden],
        vec![web_entry.clone()],
        [package.clone(), web_entry.clone()],
        vec![],
    );
    let workspace = crate::codebase::workspaces::IndexedWorkspaceMap::from_packages(vec![
        crate::codebase::workspaces::WorkspacePackage {
            name: "@x/web".to_string(),
            dir: root.join("packages/web"),
            entry: Some(web_entry.clone()),
            exports: None,
            imports: None,
        },
        crate::codebase::workspaces::WorkspacePackage {
            name: "@x/hidden".to_string(),
            dir: root.join("hidden"),
            entry: Some(root.join("hidden/index.ts")),
            exports: None,
            imports: None,
        },
    ])
    .with_manifest_dependency_names(package.clone(), vec!["@x/web".to_string()]);

    let empty_imports = Vec::new();
    assert!(collect_workspace_edges(
        &empty_imports,
        &resolver,
        &Default::default(),
        &graph_files,
        &crate::codebase::analysis_session::PathInterner::new()
    )
    .is_empty());
    let imports_path = root.join("packages/api/src/users.mts");
    let imports_items = vec![ExtractedImport {
        specifier: "@x/web".to_string(),
        kind: ImportKind::Static,
        line: 1,
        function_scope: None,
        function_scope_id: None,
        side_effect_only: false,
        re_export: false,
        runtime_reachable: false,
        computed: false,
    }];
    let imports_facts = crate::codebase::ts_source::facts::TsFileFacts {
        imports: imports_items,
        ..Default::default()
    };
    let imports = vec![(&imports_path, &imports_facts, Default::default())];
    let edges = collect_workspace_edges(
        &imports,
        &resolver,
        &workspace,
        &graph_files,
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert_eq!(edges.len(), 1);
    let hidden_workspace_path = root.join("packages/api/src/users.mts");
    let hidden_workspace_items = vec![ExtractedImport {
        specifier: "@x/hidden".to_string(),
        kind: ImportKind::Static,
        line: 1,
        function_scope: None,
        function_scope_id: None,
        side_effect_only: false,
        re_export: false,
        runtime_reachable: false,
        computed: false,
    }];
    let hidden_workspace_facts = crate::codebase::ts_source::facts::TsFileFacts {
        imports: hidden_workspace_items,
        ..Default::default()
    };
    let hidden_workspace_import = vec![(
        &hidden_workspace_path,
        &hidden_workspace_facts,
        Default::default(),
    )];
    assert!(collect_workspace_edges(
        &hidden_workspace_import,
        &resolver,
        &workspace,
        &graph_files,
        &crate::codebase::analysis_session::PathInterner::new()
    )
    .is_empty());

    let manifest_edges = (crate::codebase::package_analysis::hooks().manifest_edges)(
        &[
            package.clone(),
            root.join("missing/package.json"),
            root.join("bad/package.json"),
            root.join("hidden/package.json"),
        ],
        &workspace,
        &graph_files,
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert!(manifest_edges.iter().any(|(_, to, kind)| {
        *kind == EdgeKind::PackageDependency && to.as_file() == Some(web_entry.as_path())
    }));

    assert_eq!(
        collect_import_edges(
            &imports,
            &resolver,
            &Default::default(),
            &graph_files,
            &crate::codebase::analysis_session::PathInterner::new()
        )
        .len(),
        1
    );
    let hidden_imports_path = root.join("packages/api/src/users.mts");
    let hidden_imports_items = vec![ExtractedImport {
        specifier: "./index.mts".to_string(),
        kind: ImportKind::Static,
        line: 1,
        function_scope: None,
        function_scope_id: None,
        side_effect_only: false,
        re_export: false,
        runtime_reachable: false,
        computed: false,
    }];
    let hidden_imports_facts = crate::codebase::ts_source::facts::TsFileFacts {
        imports: hidden_imports_items,
        ..Default::default()
    };
    let hidden_imports = vec![(
        &hidden_imports_path,
        &hidden_imports_facts,
        Default::default(),
    )];
    assert!(collect_import_edges(
        &hidden_imports,
        &resolver,
        &Default::default(),
        &graph_files,
        &crate::codebase::analysis_session::PathInterner::new()
    )
    .is_empty());
    let require_resolve_path = root.join("packages/api/src/users.mts");
    let require_resolve_items = vec![ExtractedImport {
        specifier: "@x/web".to_string(),
        kind: ImportKind::RequireResolve,
        line: 1,
        function_scope: None,
        function_scope_id: None,
        side_effect_only: false,
        re_export: false,
        runtime_reachable: false,
        computed: false,
    }];
    let require_resolve_facts = crate::codebase::ts_source::facts::TsFileFacts {
        imports: require_resolve_items,
        ..Default::default()
    };
    let require_resolve_imports = vec![(
        &require_resolve_path,
        &require_resolve_facts,
        Default::default(),
    )];
    let require_resolve_edges = collect_import_edges(
        &require_resolve_imports,
        &resolver,
        &workspace,
        &graph_files,
        &crate::codebase::analysis_session::PathInterner::new(),
    );
    assert_eq!(require_resolve_edges.len(), 1);
    assert_eq!(require_resolve_edges[0].2, EdgeKind::RequireResolve);
    assert!(collect_workspace_edges(
        &require_resolve_imports,
        &resolver,
        &workspace,
        &graph_files,
        &crate::codebase::analysis_session::PathInterner::new(),
    )
    .is_empty());
    assert_eq!(package_name_from_spec("@scope/pkg/path"), "@scope/pkg");
    assert_eq!(package_name_from_spec("@scope"), "@scope");
}
