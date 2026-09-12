#[test]
fn route_import_helpers_cover_missing_canonical_parents_and_visible_remap() {
    let missing = PathBuf::from("/no-mistakes-missing-route-import-target");
    let empty = std::collections::BTreeMap::new();
    let unresolved = route_import_resolution_source(&missing, &empty);
    assert_eq!(unresolved, missing);

    let parent = PathBuf::from("/repo/src");
    let canonical_parent = PathBuf::from("/canonical/src");
    let mut directories = std::collections::BTreeMap::new();
    directories.insert(parent.clone(), canonical_parent.clone());
    let file = parent.join("route.ts");
    assert_eq!(
        route_import_resolution_source(&file, &directories),
        canonical_parent.join("route.ts")
    );
    let _ = route_import_resolution_source(Path::new(".."), &directories);
    let _ = route_import_resolution_source(Path::new(""), &directories);

    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/simple/fixture"),
    );
    let a = root.join("a.mts");
    let b = root.join("b.mts");
    let graph_files = GraphFiles::from_parts(
        vec![a.clone(), b.clone()],
        vec![a.clone(), b.clone()],
        [a.clone(), b.clone()],
        vec![],
    );
    let mut visible_by_name = std::collections::BTreeMap::<std::ffi::OsString, Vec<PathBuf>>::new();
    for visible in graph_files.indexable() {
        if let Some(name) = visible.file_name() {
            visible_by_name
                .entry(name.to_os_string())
                .or_default()
                .push(visible.clone());
        }
    }
    assert_eq!(
        route_import_visible_target(a.clone(), &graph_files, &visible_by_name),
        Some(a.clone())
    );
    assert!(
        route_import_visible_target(missing.clone(), &graph_files, &visible_by_name).is_none()
    );
    assert!(route_import_visible_target(PathBuf::from(".."), &graph_files, &visible_by_name).is_none());

    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let session = crate::codebase::analysis_session::AnalysisSession::new(None);
    let mut facts = TsFactMap::new();
    facts.insert(
        a.clone(),
        TsFileFacts {
            imports: vec![ExtractedImport {
                specifier: "./b.mts".to_string(),
                kind: ImportKind::Static,
                line: 1,
                function_scope: None,
                function_scope_id: None,
                side_effect_only: false,
                re_export: false,
                runtime_reachable: false,
            }],
            ..TsFileFacts::default()
        },
    );
    let edges = collect_route_import_edges(
        std::slice::from_ref(&a),
        &facts,
        &tsconfig,
        None,
        &graph_files,
        &session,
    );
    assert!(
        edges
            .iter()
            .any(|(_, _, kind)| *kind == EdgeKind::RouteImport),
        "{edges:#?}"
    );

    let parse_error_facts = {
        let mut map = TsFactMap::new();
        map.insert(
            a.clone(),
            TsFileFacts {
                parse_error: Some("synthetic".to_string()),
                imports: vec![ExtractedImport {
                    specifier: "./b.mts".to_string(),
                    kind: ImportKind::Static,
                    line: 1,
                    function_scope: None,
                    function_scope_id: None,
                    side_effect_only: false,
                    re_export: false,
                    runtime_reachable: false,
                }],
                ..TsFileFacts::default()
            },
        );
        map
    };
    let skipped = collect_route_import_edges(
        std::slice::from_ref(&a),
        &parse_error_facts,
        &tsconfig,
        None,
        &graph_files,
        &session,
    );
    assert!(skipped.is_empty(), "{skipped:#?}");
}

#[cfg(unix)]
#[test]
fn route_import_resolution_follows_symlink_files_and_broken_links() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link"),
    );
    let client = root.join("src/route-client.ts");
    let empty = std::collections::BTreeMap::new();
    let _ = route_import_resolution_source(&client, &empty);
    let _ = route_import_resolution_source(&root, &empty);

    let graph_files = GraphFiles::discover(&root);
    let mut catalog_visible = graph_files.all().to_vec();
    catalog_visible.push(root.join("tsconfig.json"));
    let catalog = crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
        &root,
        std::slice::from_ref(&root),
        &catalog_visible,
    );
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json"))
        .expect("symlink workspace tsconfig loads");
    let session = crate::codebase::analysis_session::AnalysisSession::new(None);
    let facts = collect_ts_facts(graph_files.indexable(), TsFactPlan::imports());
    let with_catalog = collect_route_import_edges(
        graph_files.indexable(),
        &facts,
        &tsconfig,
        Some(&catalog),
        &graph_files,
        &session,
    );
    let _ = with_catalog;
}
