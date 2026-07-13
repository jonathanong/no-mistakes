#[test]
fn graph_build_plan_from_allowed_covers_each_edge_family() {
    assert!(!GraphBuildPlan::test_impact().route_imports);
    assert!(GraphBuildPlan::test_impact().playwright_selectors);
    assert!(GraphBuildPlan::all().imports);
    assert!(!GraphBuildPlan::all().route_imports);
    assert!(GraphBuildPlan::all().workspace);
    assert_eq!(GraphBuildPlan::from_allowed(None), GraphBuildPlan::all());

    let route_import_only: HashSet<_> = [EdgeKind::RouteImport].into();
    assert!(GraphBuildPlan::from_allowed(Some(&route_import_only)).route_imports);

    let allowed: HashSet<_> = [
        EdgeKind::TypeImport,
        EdgeKind::WorkspaceImport,
        EdgeKind::TestOf,
        EdgeKind::MarkdownLink,
        EdgeKind::CiInvocation,
        EdgeKind::RouteRef,
        EdgeKind::QueueEnqueue,
        EdgeKind::QueueWorker,
        EdgeKind::RouteTest,
        EdgeKind::Layout,
        EdgeKind::HttpCall,
        EdgeKind::ProcessSpawn,
        EdgeKind::AssetImport,
        EdgeKind::ReactRender,
    ]
    .into();
    let plan = GraphBuildPlan::from_allowed(Some(&allowed));
    assert!(plan.imports);
    assert!(plan.workspace);
    assert!(plan.tests);
    assert!(plan.markdown);
    assert!(plan.ci);
    assert!(plan.routes);
    assert!(plan.queues);
    assert!(plan.playwright_routes);
    assert!(plan.http);
    assert!(plan.process);
    assert!(plan.assets);
    assert!(plan.react);

    let import_only: HashSet<_> = [EdgeKind::Require].into();
    let plan = GraphBuildPlan::from_allowed(Some(&import_only));
    assert!(plan.imports);
    assert!(!plan.workspace);
    assert!(!plan.tests);
    assert!(!plan.markdown);
    assert!(!plan.ci);
    assert!(!plan.routes);
    assert!(!plan.queues);
    assert!(!plan.playwright_routes);
    assert!(!plan.http);
    assert!(!plan.process);
    assert!(!plan.assets);
    assert!(!plan.react);
}

#[test]
fn fact_lookup_defaults_and_sparse_fallback_are_complete() {
    struct MinimalFacts(TsFactMap);

    impl TsFactLookup for MinimalFacts {
        fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
            self.0.get(path)
        }
    }

    let primary_path = p("/repo/primary.ts");
    let fallback_path = p("/repo/fallback.ts");
    let primary = TsFactMap::from([(primary_path.clone(), TsFileFacts::default())]);
    let fallback = TsFactMap::from([(fallback_path.clone(), TsFileFacts::default())]);
    let minimal = MinimalFacts(primary);

    assert!(!minimal.covers_ts_fact_plan(TsFactPlan::imports()));
    assert!(minimal.graph_files().is_none());

    for prefer_fallback in [false, true] {
        let lookup = FallbackTsFactLookup {
            primary: &minimal,
            fallback: &fallback,
            prefer_fallback,
        };
        assert!(lookup.get_ts_facts(&primary_path).is_some());
        assert!(lookup.get_ts_facts(&fallback_path).is_some());
        assert!(lookup.covers_ts_fact_plan(TsFactPlan::imports()));
    }
}

#[test]
fn route_import_edges_are_runtime_only_and_do_not_prune_uncalled_functions() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/nextjs-selectors/frontend-tsconfig/fixture")
        .canonicalize()
        .expect("fixture root resolves");
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&root.join("web/tsconfig.json"))
        .expect("frontend tsconfig loads");
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            route_imports: true,
            ..Default::default()
        },
    )
    .expect("route-import graph builds");
    let allowed: HashSet<_> = [EdgeKind::RouteImport].into();
    let dependencies = graph.deps_of(
        &[NodeId::File(root.join("web/app/page.tsx"))],
        None,
        Some(&allowed),
    );
    let files = dependencies
        .iter()
        .filter_map(|entry| entry.node.as_file())
        .collect::<HashSet<_>>();

    assert!(files.contains(root.join("web/app/components/wrapped-button.tsx").as_path()));
    assert!(files.contains(
        root.join("web/app/components/wrapped-template-button.tsx")
            .as_path()
    ));
    assert!(!files.contains(root.join("web/app/components/required-button.tsx").as_path()));
}

#[test]
fn route_import_edges_fill_present_but_sparse_check_facts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/nextjs-selectors/frontend-tsconfig/fixture")
        .canonicalize()
        .expect("fixture root resolves");
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&root.join("web/tsconfig.json"))
        .expect("frontend tsconfig loads");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sparse = crate::codebase::check_facts::collect_check_facts(
        &root,
        files.clone(),
        crate::codebase::check_facts::CheckFactPlan::default(),
    );
    let graph = DepGraph::build_with_plan_file_list_and_check_facts(
        &root,
        &tsconfig,
        GraphBuildPlan {
            route_imports: true,
            ..Default::default()
        },
        files,
        &sparse,
    );
    let allowed: HashSet<_> = [EdgeKind::RouteImport].into();
    let dependencies = graph.deps_of(
        &[NodeId::File(root.join("web/app/page.tsx"))],
        None,
        Some(&allowed),
    );

    assert!(dependencies.iter().any(|entry| {
        entry.node.as_file()
            == Some(root.join("web/app/components/wrapped-button.tsx").as_path())
    }));
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

    assert!(collect_route_import_edges(
        std::slice::from_ref(&source),
        &facts,
        &tsconfig,
        &graph_files,
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
    );
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
