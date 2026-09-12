fn extracted(specifier: &str, kind: ImportKind) -> ExtractedImport {
    ExtractedImport {
        specifier: specifier.to_string(),
        kind,
        line: 1,
        function_scope: None,
        function_scope_id: None,
        side_effect_only: false,
        re_export: false,
        runtime_reachable: false,
    }
}

fn simple_fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/simple/fixture"),
    )
}

#[test]
fn import_neighbors_cover_prepared_facts_source_store_and_classifications() {
    let root = simple_fixture_root();
    let a = root.join("a.mts");
    let b = root.join("b.mts");
    let missing = root.join("missing.mts");
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph_files = GraphFiles::from_parts(
        vec![a.clone(), b.clone()],
        vec![a.clone(), b.clone()],
        [a.clone(), b.clone()],
        vec![],
    );
    let context = TsFactContext::new(&root);
    let session = crate::codebase::analysis_session::AnalysisSession::new(None);
    let resolver = crate::codebase::ts_resolver::ImportResolver::new_in_session(
        &tsconfig,
        Some(&graph_files),
        &session,
    );
    let workspace = crate::codebase::workspaces::IndexedWorkspaceMap::default();
    let mut prepared = TsFactMap::new();
    prepared.insert(
        a.clone(),
        TsFileFacts {
            imports: vec![
                extracted("./b.mts", ImportKind::Static),
                extracted("./b.mts", ImportKind::Type),
                extracted("./b.mts", ImportKind::Dynamic),
                extracted("./b.mts", ImportKind::Require),
                extracted("./b.mts", ImportKind::RequireResolve),
                extracted("./c.mts", ImportKind::Static),
                extracted("lodash", ImportKind::Static),
                extracted("./missing.mts", ImportKind::Static),
            ],
            parse_error: Some("keep prepared facts even with a parse diagnostic".to_string()),
            ..TsFileFacts::default()
        },
    );
    let type_only = HashSet::from([EdgeKind::TypeImport, EdgeKind::WorkspaceTypeImport]);

    let (prepared_neighbors, collected) = import_neighbors(
        &a,
        &resolver,
        &workspace,
        &graph_files,
        Some(&type_only),
        LazyImportFacts::new(Some(&prepared), TsFactPlan::imports(), &context),
        &session,
    );
    assert!(collected.is_none(), "prepared facts must skip a second parse");
    let _ = prepared_neighbors;

    let (unfiltered, _) = import_neighbors(
        &a,
        &resolver,
        &workspace,
        &graph_files,
        None,
        LazyImportFacts::new(Some(&prepared), TsFactPlan::imports(), &context),
        &session,
    );
    assert!(
        unfiltered
            .iter()
            .any(|(node, kind)| node.as_file() == Some(b.as_path())
                && matches!(
                    kind,
                    EdgeKind::WorkspaceImport
                        | EdgeKind::Import
                        | EdgeKind::WorkspaceTypeImport
                        | EdgeKind::TypeImport
                        | EdgeKind::Require
                        | EdgeKind::RequireResolve
                )),
        "{unfiltered:#?}"
    );

    let mut source_plan = TsFactPlan::imports();
    source_plan.source = true;
    let (from_disk, collected) = import_neighbors(
        &a,
        &resolver,
        &workspace,
        &graph_files,
        None,
        LazyImportFacts::new(None, source_plan, &context),
        &session,
    );
    assert!(collected.is_some());
    assert!(
        from_disk
            .iter()
            .any(|(node, _)| node.as_file() == Some(b.as_path())),
        "{from_disk:#?}"
    );

    let inventory = std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(
        std::slice::from_ref(&missing),
    ));
    let sources = crate::codebase::ts_source::SourceStore::new(inventory);
    let (missing_neighbors, collected) = import_neighbors(
        &missing,
        &resolver,
        &workspace,
        &graph_files,
        None,
        LazyImportFacts::new(None, TsFactPlan::imports(), &context).with_source_store(&sources),
        &session,
    );
    assert!(missing_neighbors.is_empty());
    assert!(
        collected
            .as_ref()
            .and_then(|facts| facts.parse_error.as_ref())
            .is_some(),
        "{collected:#?}"
    );

    let session_missing = import_neighbors(
        &missing,
        &resolver,
        &workspace,
        &graph_files,
        None,
        LazyImportFacts::new(None, TsFactPlan::imports(), &context),
        &session,
    );
    assert!(session_missing.0.is_empty());
    assert!(session_missing.1.and_then(|facts| facts.parse_error).is_some());
}
