#[test]
fn graph_collectors_cover_defensive_empty_and_error_paths() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("codebase-intel"));
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let graph_files = GraphFiles::from_parts(vec![], vec![], Vec::<PathBuf>::new(), vec![]);
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let fact_context = TsFactContext::default();

    assert!(
        lazy_import_deps_of_with_files(
            &[NodeId::file(root.join("packages/api/src/index.mts"))],
            &root,
            &tsconfig,
            None,
            &graph_files,
            None,
        )
        .is_empty()
    );
    assert!(
        import_neighbors(
            &root.join("missing.mts"),
            &crate::codebase::ts_resolver::ImportResolver::new(&tsconfig),
            &crate::codebase::workspaces::IndexedWorkspaceMap::default(),
            &graph_files,
            None,
            LazyImportFacts::new(None, TsFactPlan::imports(), &fact_context),
            &session,
        )
        .0
        .is_empty()
    );

    assert!(
        collect_workspace_manifest_edges(
            &[root.join("missing/package.json")],
            &crate::codebase::workspaces::IndexedWorkspaceMap::from_packages(vec![
                crate::codebase::workspaces::WorkspacePackage {
                    name: "@x/missing".to_string(),
                    dir: root.join("packages/missing"),
                    entry: Some(root.join("packages/missing/index.ts")),
                    exports: None,
                    imports: None,
                },
            ]),
            &graph_files,
            &crate::codebase::analysis_session::PathInterner::new()
        )
        .is_empty()
    );
    assert!(
        collect_test_edges(
            Path::new("."),
            &[PathBuf::from("/")],
            None,
            &crate::codebase::analysis_session::PathInterner::new()
        )
        .is_empty()
    );
    assert!(
        collect_test_edges(
            Path::new("."),
            &[PathBuf::from("no-parent.ts")],
            None,
            &crate::codebase::analysis_session::PathInterner::new()
        )
        .is_empty()
    );
    assert!(
        collect_md_edges(
            &[PathBuf::from("/")],
            &graph_files,
            &crate::codebase::analysis_session::PathInterner::new(),
            None,
        )
        .is_empty()
    );
    assert!(
        collect_md_edges(
            &[PathBuf::from("README.md")],
            &graph_files,
            &crate::codebase::analysis_session::PathInterner::new(),
            None,
        )
        .is_empty()
    );

    let mut forward = EdgeMap::default();
    let mut reverse = EdgeMap::default();
    let parsed = parsed_workflow_set(&root.join("missing"), &[]);
    add_ci_edges(
        &root.join("missing"),
        &[],
        &parsed,
        &mut forward,
        &mut reverse,
        &crate::codebase::analysis_session::PathInterner::new(),
        None,
    );
    assert!(forward.is_empty());

    assert!(
        collect_route_edges(
            &root.join("missing"),
            &tsconfig,
            &crate::codebase::ts_resolver::ImportResolver::new(&tsconfig),
            &[],
            None,
            None,
        )
        .is_empty()
    );
    test_support::add_queue_edges(
        &root.join("missing"),
        &crate::codebase::ts_resolver::ImportResolver::new(&tsconfig),
        &[],
        None,
        None,
        &mut forward,
        &mut reverse,
    );
    assert!(
        collect_http_call_edges(
            &root.join("missing"),
            None,
            &[],
            &[],
            &[],
            None,
            &crate::codebase::analysis_session::PathInterner::new()
        )
        .is_empty()
    );
}

#[test]
fn lazy_import_facts_memoize_parse_errors() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/selector-malformed-app-source/fixture"),
    );
    let malformed = root.join("web/components/save-button.tsx");
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph_files = GraphFiles::from_parts(
        vec![malformed.clone()],
        vec![malformed.clone()],
        [malformed.clone()],
        vec![],
    );
    let context = TsFactContext::new(&root);
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();

    let (neighbors, collected) = import_neighbors(
        &malformed,
        &crate::codebase::ts_resolver::ImportResolver::new(&tsconfig),
        &crate::codebase::workspaces::IndexedWorkspaceMap::default(),
        &graph_files,
        None,
        LazyImportFacts::new(None, TsFactPlan::imports(), &context),
        &session,
    );

    assert!(neighbors.is_empty());
    assert!(
        collected
            .and_then(|facts| facts.parse_error)
            .is_some_and(|error| error.contains("failed to parse"))
    );
}

#[cfg(unix)]
#[test]
fn markdown_links_remap_canonical_targets_to_the_visible_spelling() {
    let via_link = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/markdown-canonical-link/link/src/value.ts"),
    );
    let notes = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/markdown-canonical-link/real/src/notes.md"),
    );
    let files = GraphFiles::from_files(vec![via_link.clone()]);
    let edges = collect_md_edges(
        std::slice::from_ref(&notes),
        &files,
        &crate::codebase::analysis_session::PathInterner::new(),
        None,
    );
    assert_eq!(
        edges,
        vec![(
            NodeId::file(notes),
            NodeId::file(via_link),
            EdgeKind::MarkdownLink,
        )]
    );
}

#[test]
fn lazy_import_neighbors_read_through_a_source_store_and_typed_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("simple"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let a = root.join("a.mts");
    let b = root.join("b.mts");
    let graph_files = GraphFiles::from_files(vec![a.clone(), b.clone()]);
    let context = TsFactContext::new(&root);
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let inventory = crate::codebase::ts_source::FileInventory::from_paths(&[a.clone(), b.clone()]);
    let sources = crate::codebase::ts_source::SourceStore::new(std::sync::Arc::new(inventory));
    let (neighbors, collected) = import_neighbors(
        &a,
        &crate::codebase::ts_resolver::ImportResolver::new(&tsconfig),
        &crate::codebase::workspaces::IndexedWorkspaceMap::default(),
        &graph_files,
        None,
        LazyImportFacts::new(None, TsFactPlan::imports(), &context).with_source_store(&sources),
        &session,
    );
    assert!(!neighbors.is_empty() || collected.is_some());

    let facts = TsFactMap::from([(
        a.clone(),
        TsFileFacts {
            imports: vec![
                ExtractedImport {
                    specifier: "./b".to_string(),
                    kind: ImportKind::Type,
                    line: 1,
                    function_scope: None,
                    side_effect_only: false,
                    re_export: false,
                    runtime_reachable: false,
                },
                ExtractedImport {
                    specifier: "./b".to_string(),
                    kind: ImportKind::RequireResolve,
                    line: 2,
                    function_scope: None,
                    side_effect_only: false,
                    re_export: false,
                    runtime_reachable: true,
                },
            ],
            ..TsFileFacts::default()
        },
    )]);
    let allowed = std::collections::HashSet::from([
        EdgeKind::WorkspaceTypeImport,
        EdgeKind::RequireResolve,
        EdgeKind::WorkspaceImport,
    ]);
    let (typed, _) = import_neighbors(
        &a,
        &crate::codebase::ts_resolver::ImportResolver::new(&tsconfig),
        &crate::codebase::workspaces::IndexedWorkspaceMap::default(),
        &graph_files,
        Some(&allowed),
        LazyImportFacts::new(Some(&facts), TsFactPlan::imports(), &context),
        &session,
    );
    assert!(typed.iter().any(|(_, kind)| {
        matches!(
            kind,
            EdgeKind::WorkspaceTypeImport | EdgeKind::RequireResolve
        )
    }));
}

#[test]
fn lazy_import_neighbors_parse_typed_and_require_resolve_without_prepared_facts() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("lazy-import-kinds"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let a = root.join("a.mts");
    let b = root.join("b.mts");
    let graph_files = GraphFiles::from_files(vec![a.clone(), b.clone()]);
    let context = TsFactContext::new(&root);
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let allowed = std::collections::HashSet::from([
        EdgeKind::WorkspaceTypeImport,
        EdgeKind::RequireResolve,
        EdgeKind::WorkspaceImport,
        EdgeKind::Import,
        EdgeKind::TypeImport,
        EdgeKind::Require,
    ]);
    let (neighbors, collected) = import_neighbors(
        &a,
        &crate::codebase::ts_resolver::ImportResolver::new(&tsconfig),
        &crate::codebase::workspaces::IndexedWorkspaceMap::default(),
        &graph_files,
        Some(&allowed),
        LazyImportFacts::new(None, TsFactPlan::imports(), &context),
        &session,
    );
    assert!(collected.is_some());
    assert!(
        !neighbors.is_empty()
            || collected
                .as_ref()
                .is_some_and(|facts| !facts.imports.is_empty()),
        "{neighbors:#?} {collected:#?}"
    );
    let none_allowed = std::collections::HashSet::new();
    let (filtered, _) = import_neighbors(
        &a,
        &crate::codebase::ts_resolver::ImportResolver::new(&tsconfig),
        &crate::codebase::workspaces::IndexedWorkspaceMap::default(),
        &graph_files,
        Some(&none_allowed),
        LazyImportFacts::new(None, TsFactPlan::imports(), &context),
        &session,
    );
    assert!(filtered.is_empty());
}

#[test]
fn route_import_edges_resolve_through_an_unbounded_catalog() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("simple"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let a = root.join("a.mts");
    let b = root.join("b.mts");
    let facts = TsFactMap::from([(
        a.clone(),
        TsFileFacts {
            imports: vec![ExtractedImport {
                specifier: "./b.mts".to_string(),
                kind: ImportKind::Static,
                line: 1,
                function_scope: None,
                side_effect_only: false,
                re_export: false,
                runtime_reachable: true,
            }],
            ..TsFileFacts::default()
        },
    )]);
    let graph_files = GraphFiles::from_files(vec![a.clone(), b.clone()]);
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig.clone(), None);
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let edges = collect_route_import_edges(
        std::slice::from_ref(&a),
        &facts,
        &tsconfig,
        Some(&catalog),
        &graph_files,
        &session,
    );
    assert!(
        edges
            .iter()
            .any(|(from, to, kind)| *kind == EdgeKind::RouteImport
                && from.as_file() == Some(a.as_path())
                && to.as_file() == Some(b.as_path())),
        "{edges:#?}"
    );
}
