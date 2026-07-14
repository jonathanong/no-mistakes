#[test]
fn prepared_graph_rejects_an_incomplete_fact_plan_without_refilling() {
    let fixture = materialized_frontend_tsconfig_fixture();
    let root = fixture.path().canonicalize().unwrap();
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&root.join("web/tsconfig.json"))
        .expect("frontend tsconfig loads");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files.clone(),
        crate::codebase::check_facts::CheckFactPlan::default(),
    );

    crate::ast::begin_parse_count(&root);
    let result = DepGraph::build_with_plan_file_list_config_and_complete_check_facts(
        &root,
        &tsconfig,
        GraphBuildPlan {
            route_imports: true,
            ..Default::default()
        },
        files,
        None,
        &facts,
    );
    let parse_counts = crate::ast::finish_parse_count(&root);
    assert!(parse_counts.is_empty(), "{parse_counts:#?}");
    let error = match result {
        Ok(_) => panic!("prepared graph must reject an incomplete fact plan"),
        Err(error) => error,
    };

    assert!(format!("{error:#}").contains("do not cover the required TS fact plan"));
}

#[test]
fn prepared_graph_rejects_missing_file_facts_without_refilling() {
    let fixture = materialized_frontend_tsconfig_fixture();
    let root = fixture.path().canonicalize().unwrap();
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&root.join("web/tsconfig.json"))
        .expect("frontend tsconfig loads");
    let plan = GraphBuildPlan {
        route_imports: true,
        ..Default::default()
    };
    let page = root.join("web/app/page.tsx");
    let wrapped = root.join("web/app/components/wrapped-button.tsx");
    let (fact_plan, fact_context) = ts_fact_plan_and_context_for_plan(&root, plan);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        vec![page.clone()],
        crate::codebase::check_facts::CheckFactPlan {
            graph: fact_plan,
            graph_context: fact_context,
            ..Default::default()
        },
    );

    crate::ast::begin_parse_count(&root);
    let result = DepGraph::build_with_plan_file_list_config_and_complete_check_facts(
        &root,
        &tsconfig,
        plan,
        vec![page, wrapped],
        None,
        &facts,
    );
    let parse_counts = crate::ast::finish_parse_count(&root);
    assert!(parse_counts.is_empty(), "{parse_counts:#?}");
    let error = match result {
        Ok(_) => panic!("prepared graph must reject missing file facts"),
        Err(error) => error,
    };

    assert!(format!("{error:#}").contains("missing 1 indexable file"));
}

#[test]
fn prepared_graph_rejects_a_mismatched_fact_universe_without_refilling() {
    let fixture = materialized_frontend_tsconfig_fixture();
    let root = fixture.path().canonicalize().unwrap();
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&root.join("web/tsconfig.json"))
        .expect("frontend tsconfig loads");
    let plan = GraphBuildPlan {
        route_imports: true,
        ..Default::default()
    };
    let page = root.join("web/app/page.tsx");
    let wrapped = root.join("web/app/components/wrapped-button.tsx");
    let primary_universe = vec![page.clone(), wrapped];
    let (fact_plan, fact_context) = ts_fact_plan_and_context_for_plan(&root, plan);
    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_and_playwright(
            &root,
            primary_universe.clone(),
            primary_universe,
            crate::codebase::check_facts::CheckFactPlan {
                graph: fact_plan,
                graph_context: fact_context,
                ..Default::default()
            },
            None,
        );

    crate::ast::begin_parse_count(&root);
    let result = DepGraph::build_with_plan_file_list_config_and_complete_check_facts(
        &root,
        &tsconfig,
        plan,
        vec![page],
        None,
        &facts,
    );
    let parse_counts = crate::ast::finish_parse_count(&root);
    assert!(parse_counts.is_empty(), "{parse_counts:#?}");
    let error = match result {
        Ok(_) => panic!("prepared graph must reject a mismatched fact universe"),
        Err(error) => error,
    };

    assert!(format!("{error:#}").contains("fact universe does not match"));
}

#[test]
fn route_import_resolution_tolerates_missing_source_directories() {
    // Keep this disk-missing source: it covers configured/external graph files
    // disappearing between discovery and conservative edge construction.
    let source = PathBuf::from("/no-mistakes-missing-route-import/source.ts");
    let facts = TsFactMap::from([(
        source.clone(),
        TsFileFacts {
            imports: vec![ExtractedImport {
                specifier: "./target".to_string(),
                kind: ImportKind::Static,
                line: 1,
                function_scope: None,
                side_effect_only: true,
                re_export: false,
                runtime_reachable: false,
            }],
            ..TsFileFacts::default()
        },
    )]);
    let graph_files = GraphFiles::from_files(vec![source.clone()]);
    let tsconfig = TsConfig {
        dir: PathBuf::from("/no-mistakes-missing-route-import"),
        paths_dir: PathBuf::from("/no-mistakes-missing-route-import"),
        ..TsConfig::default()
    };

    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    assert!(collect_route_import_edges(
        std::slice::from_ref(&source),
        &facts,
        &tsconfig,
        &graph_files,
        &session,
    )
    .is_empty());
    assert_eq!(
        route_import_resolution_source(Path::new("/"), &Default::default()),
        PathBuf::from("/")
    );
}

#[cfg(unix)]
#[test]
fn route_import_resolution_tolerates_broken_source_symlink() {
    // This tracked fixture must stay broken to exercise canonicalize failure.
    let broken = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/tests-impact/fixture/broken.test.mts");

    assert_eq!(
        route_import_resolution_source(&broken, &Default::default()),
        broken
    );
}

#[cfg(unix)]
#[test]
fn route_import_resolution_uses_direct_symlink_target() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/scan-config/symlinked-default-playwright/fixture")
        .canonicalize()
        .expect("fixture root resolves");
    let symlink = root.join("playwright.config.ts");
    let resolved = route_import_resolution_source(&symlink, &Default::default());

    assert_eq!(resolved, root.join("configs/shared.playwright.config.ts"));

    let real_target = root.join("configs/shared.playwright.config.ts");
    let graph_files = GraphFiles::from_files(vec![symlink.clone()]);
    let visible_by_name = std::collections::BTreeMap::from([(
        real_target
            .file_name()
            .expect("target has a name")
            .to_os_string(),
        vec![symlink.clone()],
    )]);
    assert_eq!(
        route_import_visible_target(real_target, &graph_files, &visible_by_name),
        Some(symlink)
    );
}

#[cfg(unix)]
#[test]
fn route_import_edges_resolve_from_direct_symlink_target() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/scan-config/symlinked-default-playwright/fixture")
        .canonicalize()
        .expect("fixture root resolves");
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let files = vec![
        root.join("playwright.config.ts"),
        root.join("configs/route-helper.ts"),
        root.join("configs/shared.playwright.config.ts"),
    ];
    let graph_files = GraphFiles::from_files(files);
    let graph = DepGraph::build_with_plan_files_config_and_facts(
        &root,
        &tsconfig,
        GraphBuildPlan {
            route_imports: true,
            ..Default::default()
        },
        &graph_files,
        None,
        None,
    )
    .expect("symlink route-import graph builds");
    let allowed = HashSet::from([EdgeKind::RouteImport]);
    let dependencies = graph.deps_of(
        &[NodeId::File(root.join("playwright.config.ts"))],
        None,
        Some(&allowed),
    );

    assert!(dependencies.iter().any(|entry| {
        entry.node.as_file() == Some(root.join("configs/route-helper.ts").as_path())
    }));
}
