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
    assert_eq!(
        route_import_resolution_source(&parent.join("."), &directories),
        parent.join(".")
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

    let nameless = PathBuf::from(".");
    let nameless_files = GraphFiles::from_parts(
        vec![a.clone(), nameless.clone()],
        vec![a.clone(), nameless.clone()],
        [a.clone(), nameless.clone()],
        vec![],
    );
    let _ = collect_route_import_edges(
        std::slice::from_ref(&a),
        &facts,
        &tsconfig,
        None,
        &nameless_files,
        &session,
    );

    let missing_file = PathBuf::from("/no-mistakes-missing-route-import-target/file.ts");
    let mut missing_facts = TsFactMap::new();
    missing_facts.insert(
        missing_file.clone(),
        TsFileFacts {
            imports: vec![ExtractedImport {
                specifier: "./b.mts".to_string(),
                kind: ImportKind::Dynamic,
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
    let missing_files = GraphFiles::from_parts(
        vec![missing_file.clone()],
        vec![missing_file.clone()],
        [missing_file.clone()],
        vec![],
    );
    let skipped_missing = collect_route_import_edges(
        std::slice::from_ref(&missing_file),
        &missing_facts,
        &tsconfig,
        None,
        &missing_files,
        &session,
    );
    assert!(skipped_missing.is_empty(), "{skipped_missing:#?}");
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

    let real_helper = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/real/src/route-helper.ts"),
    );
    let link_helper = root.join("src/route-helper.ts");
    let remap_files = GraphFiles::from_parts(
        vec![link_helper.clone()],
        vec![link_helper.clone()],
        [link_helper.clone()],
        vec![],
    );
    let mut visible_by_name = std::collections::BTreeMap::<std::ffi::OsString, Vec<PathBuf>>::new();
    visible_by_name.insert(
        std::ffi::OsString::from("route-helper.ts"),
        vec![link_helper.clone()],
    );
    let remapped = route_import_visible_target(real_helper, &remap_files, &visible_by_name);
    assert_eq!(remapped.as_deref(), Some(link_helper.as_path()));

    let file_link = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig/fixed-root-fast-path/root/src/external.ts");
    let real_external = file_link.canonicalize().expect("file symlink target exists");
    let _ = route_import_resolution_source(&file_link, &empty);
    let broken = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/finite-set-consistency/path-regex-broken-symlink/delta"),
    );
    let _ = route_import_resolution_source(&broken, &empty);

    let file_link_files = GraphFiles::from_parts(
        vec![file_link.clone()],
        vec![file_link.clone()],
        [file_link.clone()],
        vec![],
    );
    let mut file_link_names = std::collections::BTreeMap::<std::ffi::OsString, Vec<PathBuf>>::new();
    file_link_names.insert(
        std::ffi::OsString::from("external.ts"),
        vec![file_link.clone()],
    );
    let remapped_file = route_import_visible_target(real_external.clone(), &file_link_files, &file_link_names);
    assert_eq!(remapped_file.as_deref(), Some(file_link.as_path()));
}
