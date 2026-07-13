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
    let graph_visible = HashSet::from([fallback_path.clone()]);

    assert!(!minimal.covers_ts_fact_plan(TsFactPlan::imports()));
    assert!(minimal.graph_files().is_none());

    for prefer_fallback in [false, true] {
        let lookup = FallbackTsFactLookup::new(
            &minimal,
            &fallback,
            prefer_fallback,
            std::slice::from_ref(&fallback_path),
            &graph_visible,
        );
        assert!(lookup.get_ts_facts(&primary_path).is_some());
        assert!(lookup.get_ts_facts(&fallback_path).is_some());
        assert!(lookup.covers_ts_fact_plan(TsFactPlan::imports()));
        assert_eq!(lookup.graph_files(), Some([fallback_path.clone()].as_slice()));
        assert!(lookup.get_playwright_facts(&primary_path).is_none());
        assert!(lookup
            .get_or_compute_app_selector_occurrences(false, &|| Ok(Vec::new()))
            .expect("selector occurrences compute")
            .is_empty());
        assert!(lookup
            .get_or_compute_playwright_routes(&|| Vec::new())
            .is_empty());
        assert!(lookup
            .get_or_compute_app_text_targets(&|| Ok(Vec::new()))
            .expect("app text targets compute")
            .is_empty());
        assert!(lookup
            .get_or_compute_route_reachable_files(&|| Ok(Default::default()))
            .expect("route reachability computes")
            .is_empty());
    }
}

#[test]
fn graph_universe_comparison_rejects_duplicate_false_matches() {
    assert!(!same_graph_universe(
        &[p("/repo/a.ts"), p("/repo/a.ts")],
        &HashSet::from([p("/repo/a.ts"), p("/repo/b.ts")]),
    ));
}

#[test]
fn sparse_fallback_preserves_check_fact_playwright_data_and_caches() {
    use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts, PlaywrightTestFacts};
    use std::cell::Cell;
    use std::sync::Arc;

    let primary_path = p("/repo/primary.ts");
    let fallback_path = p("/repo/fallback.ts");
    let mut primary = CheckFactMap {
        files: vec![primary_path.clone(), fallback_path.clone()],
        ..CheckFactMap::default()
    };
    primary.ts.insert(
        primary_path.clone(),
        CheckFileFacts {
            playwright: Some(PlaywrightTestFacts {
                urls: Vec::new(),
                selectors: Vec::new(),
                text_locators: Vec::new(),
                helper_references: Vec::new(),
            }),
            ..CheckFileFacts::default()
        },
    );
    let fallback = TsFactMap::from([(fallback_path.clone(), TsFileFacts::default())]);
    let graph_files = [fallback_path.clone(), primary_path.clone()];
    let graph_visible = HashSet::from(graph_files.clone());
    let lookup = FallbackTsFactLookup::new(
        &primary,
        &fallback,
        true,
        &graph_files,
        &graph_visible,
    );

    assert_eq!(lookup.graph_files(), Some(graph_files.as_slice()));
    assert!(lookup.get_playwright_facts(&primary_path).is_some());
    assert!(lookup.get_ts_facts(&fallback_path).is_some());

    let selector_calls = Cell::new(0);
    let selectors = || {
        selector_calls.set(selector_calls.get() + 1);
        Ok(Vec::new())
    };
    let first = lookup
        .get_or_compute_app_selector_occurrences(false, &selectors)
        .expect("selector occurrences compute");
    let second = lookup
        .get_or_compute_app_selector_occurrences(false, &selectors)
        .expect("selector occurrences cache");
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(selector_calls.get(), 1);

    let route_calls = Cell::new(0);
    let routes = || {
        route_calls.set(route_calls.get() + 1);
        Vec::new()
    };
    let first = lookup.get_or_compute_playwright_routes(&routes);
    let second = lookup.get_or_compute_playwright_routes(&routes);
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(route_calls.get(), 1);

    let text_calls = Cell::new(0);
    let text_targets = || {
        text_calls.set(text_calls.get() + 1);
        Ok(Vec::new())
    };
    let first = lookup
        .get_or_compute_app_text_targets(&text_targets)
        .expect("text targets compute");
    let second = lookup
        .get_or_compute_app_text_targets(&text_targets)
        .expect("text targets cache");
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(text_calls.get(), 1);

    let reachability_calls = Cell::new(0);
    let reachability = || {
        reachability_calls.set(reachability_calls.get() + 1);
        Ok(Default::default())
    };
    let first = lookup
        .get_or_compute_route_reachable_files(&reachability)
        .expect("route reachability compute");
    let second = lookup
        .get_or_compute_route_reachable_files(&reachability)
        .expect("route reachability cache");
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(reachability_calls.get(), 1);
}

#[test]
fn sparse_fallback_isolates_playwright_caches_for_a_different_graph_universe() {
    use crate::codebase::check_facts::CheckFactMap;
    use std::cell::Cell;

    let primary_path = p("/repo/primary.ts");
    let graph_path = p("/repo/current-graph.ts");
    let mut primary = CheckFactMap::default();
    primary.files.push(primary_path);
    let fallback = TsFactMap::from([(graph_path.clone(), TsFileFacts::default())]);
    let graph_visible = HashSet::from([graph_path.clone()]);

    let primary_selector_calls = Cell::new(0);
    primary
        .get_or_compute_app_selector_occurrences(false, &|| {
            primary_selector_calls.set(primary_selector_calls.get() + 1);
            Ok(Vec::new())
        })
        .expect("primary selectors compute");
    let primary_route_calls = Cell::new(0);
    primary.get_or_compute_playwright_routes(&|| {
        primary_route_calls.set(primary_route_calls.get() + 1);
        Vec::new()
    });
    let primary_text_calls = Cell::new(0);
    primary
        .get_or_compute_app_text_targets(&|| {
            primary_text_calls.set(primary_text_calls.get() + 1);
            Ok(Vec::new())
        })
        .expect("primary text targets compute");
    let primary_reachability_calls = Cell::new(0);
    primary
        .get_or_compute_route_reachable_files(&|| {
            primary_reachability_calls.set(primary_reachability_calls.get() + 1);
            Ok(Default::default())
        })
        .expect("primary reachability computes");

    let lookup = FallbackTsFactLookup::new(
        &primary,
        &fallback,
        true,
        std::slice::from_ref(&graph_path),
        &graph_visible,
    );
    assert_eq!(lookup.graph_files(), Some([graph_path.clone()].as_slice()));

    let selector_calls = Cell::new(0);
    for _ in 0..2 {
        lookup
            .get_or_compute_app_selector_occurrences(false, &|| {
                selector_calls.set(selector_calls.get() + 1);
                Ok(Vec::new())
            })
            .expect("isolated selectors compute");
    }
    let route_calls = Cell::new(0);
    for _ in 0..2 {
        lookup.get_or_compute_playwright_routes(&|| {
            route_calls.set(route_calls.get() + 1);
            Vec::new()
        });
    }
    let text_calls = Cell::new(0);
    let reachability_calls = Cell::new(0);
    for _ in 0..2 {
        lookup
            .get_or_compute_app_text_targets(&|| {
                text_calls.set(text_calls.get() + 1);
                Ok(Vec::new())
            })
            .expect("isolated text targets compute");
        lookup
            .get_or_compute_route_reachable_files(&|| {
                reachability_calls.set(reachability_calls.get() + 1);
                Ok(Default::default())
            })
            .expect("isolated reachability computes");
    }

    assert_eq!(selector_calls.get(), 2);
    assert_eq!(route_calls.get(), 2);
    assert_eq!(text_calls.get(), 2);
    assert_eq!(reachability_calls.get(), 2);
    assert_eq!(primary_selector_calls.get(), 1);
    assert_eq!(primary_route_calls.get(), 1);
    assert_eq!(primary_text_calls.get(), 1);
    assert_eq!(primary_reachability_calls.get(), 1);
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
