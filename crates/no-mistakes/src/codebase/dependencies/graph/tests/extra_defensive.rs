#[test]
fn graph_collectors_cover_defensive_empty_and_error_paths() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("codebase-intel"));
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let graph_files = GraphFiles::from_parts(vec![], vec![], Vec::<PathBuf>::new(), vec![]);
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let fact_context = TsFactContext::default();

    assert!(lazy_import_deps_of_with_files(
        &[NodeId::file(root.join("packages/api/src/index.mts"))],
        &root,
        &tsconfig,
        None,
        &graph_files,
        None,
    )
    .is_empty());
    assert!(import_neighbors(
        &root.join("missing.mts"),
        &crate::codebase::ts_resolver::ImportResolver::new(&tsconfig),
        &crate::codebase::workspaces::IndexedWorkspaceMap::default(),
        &graph_files,
        None,
        LazyImportFacts::new(None, TsFactPlan::imports(), &fact_context),
        &session,
    )
    .0
    .is_empty());

    assert!(collect_workspace_manifest_edges(
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
    .is_empty());
    assert!(collect_test_edges(
        Path::new("."),
        &[PathBuf::from("/")],
        None,
        &crate::codebase::analysis_session::PathInterner::new()
    )
    .is_empty());
    assert!(collect_test_edges(
        Path::new("."),
        &[PathBuf::from("no-parent.ts")],
        None,
        &crate::codebase::analysis_session::PathInterner::new()
    )
    .is_empty());
    assert!(collect_md_edges(
        &[PathBuf::from("/")],
        &graph_files,
        &crate::codebase::analysis_session::PathInterner::new(),
        None,
    )
    .is_empty());
    assert!(collect_md_edges(
        &[PathBuf::from("README.md")],
        &graph_files,
        &crate::codebase::analysis_session::PathInterner::new(),
        None,
    )
    .is_empty());

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

    assert!(collect_route_edges(
        &root.join("missing"),
        &tsconfig,
        &crate::codebase::ts_resolver::ImportResolver::new(&tsconfig),
        &[],
        None,
        None,
    )
    .is_empty());
    test_support::add_queue_edges(
        &root.join("missing"),
        &crate::codebase::ts_resolver::ImportResolver::new(&tsconfig),
        &[],
        None,
        None,
        &mut forward,
        &mut reverse,
    );
    assert!(collect_http_call_edges(
        &root.join("missing"),
        None,
        &[],
        &[],
        &[],
        None,
        &crate::codebase::analysis_session::PathInterner::new()
    )
    .is_empty());
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
    assert!(collected
        .and_then(|facts| facts.parse_error)
        .is_some_and(|error| error.contains("failed to parse")));
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
