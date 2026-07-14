fn cache_settings() -> crate::playwright::config::Settings {
    crate::playwright::config::Settings {
        frontend_root: ".".to_string(),
        playwright_configs: Vec::new(),
        project: None,
        test_include: Vec::new(),
        test_exclude: Vec::new(),
        ignore_routes: Vec::new(),
        rewrites: Vec::new(),
        navigation_helpers: Vec::new(),
        selector_attributes: Vec::new(),
        test_id_attribute_override: None,
        component_selector_attributes: std::collections::BTreeMap::new(),
        html_ids: false,
        selector_roots: vec![".".to_string()],
        selector_include: Vec::new(),
        selector_exclude: Vec::new(),
    }
}

fn materialized_frontend_tsconfig_fixture() -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/nextjs-selectors/frontend-tsconfig/fixture");
    crate::test_support::materialize_saved_fixture(&source)
}

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
    let fallback = TsFactMap::from([(
        fallback_path.clone(),
        TsFileFacts {
            parse_error: Some("fallback parse error".to_string()),
            ..TsFileFacts::default()
        },
    )]);
    let minimal = MinimalFacts(primary);
    let graph_visible = HashSet::from([fallback_path.clone()]);

    assert!(!minimal.covers_ts_fact_plan(TsFactPlan::imports()));
    assert!(minimal.graph_files().is_none());
    assert!(minimal.get_playwright_parse_error(&primary_path).is_none());
    assert!(minimal.get_playwright_fetch_facts(&primary_path).is_none());

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
            .get_playwright_parse_error(&primary_path)
            .is_none());
        assert!(lookup
            .get_or_compute_app_selector_occurrences(&cache_settings(), false, &|| Ok(Vec::new()))
            .expect("selector occurrences compute")
            .is_empty());
        assert!(lookup
            .get_or_compute_playwright_routes(&cache_settings(), &|| Vec::new())
            .is_empty());
        assert!(lookup
            .get_or_compute_app_text_targets(&cache_settings(), &|| Ok(Vec::new()))
            .expect("app text targets compute")
            .is_empty());
        assert!(lookup
            .get_or_compute_route_reachable_files(&cache_settings(), &|| Ok(Default::default()))
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
    use crate::codebase::check_facts::CheckFileFacts;
    use std::cell::Cell;
    use std::sync::Arc;

    let primary_path = p("/repo/primary.ts");
    let fallback_path = p("/repo/fallback.ts");
    let graph_files = [fallback_path.clone(), primary_path.clone()];
    let mut primary =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_and_playwright(
            Path::new("/repo"),
            vec![primary_path.clone()],
            graph_files.to_vec(),
            crate::codebase::check_facts::CheckFactPlan::default(),
            None,
        );
    primary.ts.insert(
        primary_path.clone(),
        CheckFileFacts {
            playwright: Some(
                crate::codebase::check_facts::PlaywrightTestFacts::empty(),
            ),
            ..CheckFileFacts::default()
        },
    );
    let fallback = TsFactMap::from([(
        fallback_path.clone(),
        TsFileFacts {
            parse_error: Some("fallback parse error".to_string()),
            ..TsFileFacts::default()
        },
    )]);
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
    assert_eq!(lookup.playwright_source_files(), Some([].as_slice()));
    assert!(lookup.get_playwright_test_files(None).is_none());
    let fetch_facts = lookup
        .get_playwright_fetch_facts(&fallback_path)
        .expect("fallback parse error is cached");
    let error = match fetch_facts {
        Ok(_) => panic!("fallback parse error must be retained"),
        Err(error) => error,
    };
    assert!(error.contains("fallback parse error"));

    let selector_calls = Cell::new(0);
    let selectors = || {
        selector_calls.set(selector_calls.get() + 1);
        Ok(Vec::new())
    };
    let first = lookup
        .get_or_compute_app_selector_occurrences(&cache_settings(), false, &selectors)
        .expect("selector occurrences compute");
    let second = lookup
        .get_or_compute_app_selector_occurrences(&cache_settings(), false, &selectors)
        .expect("selector occurrences cache");
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(selector_calls.get(), 1);

    let route_calls = Cell::new(0);
    let routes = || {
        route_calls.set(route_calls.get() + 1);
        Vec::new()
    };
    let first = lookup.get_or_compute_playwright_routes(&cache_settings(), &routes);
    let second = lookup.get_or_compute_playwright_routes(&cache_settings(), &routes);
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(route_calls.get(), 1);

    let text_calls = Cell::new(0);
    let text_targets = || {
        text_calls.set(text_calls.get() + 1);
        Ok(Vec::new())
    };
    let first = lookup
        .get_or_compute_app_text_targets(&cache_settings(), &text_targets)
        .expect("text targets compute");
    let second = lookup
        .get_or_compute_app_text_targets(&cache_settings(), &text_targets)
        .expect("text targets cache");
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(text_calls.get(), 1);

    let reachability_calls = Cell::new(0);
    let reachability = || {
        reachability_calls.set(reachability_calls.get() + 1);
        Ok(Default::default())
    };
    let first = lookup
        .get_or_compute_route_reachable_files(&cache_settings(), &reachability)
        .expect("route reachability compute");
    let second = lookup
        .get_or_compute_route_reachable_files(&cache_settings(), &reachability)
        .expect("route reachability cache");
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(reachability_calls.get(), 1);
}

#[test]
fn sparse_fallback_prefers_primary_playwright_fetch_errors_when_requested() {
    use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};

    let path = p("/repo/page.tsx");
    let mut primary = CheckFactMap {
        graph_files: vec![path.clone()],
        graph_files_complete: true,
        ..CheckFactMap::default()
    };
    primary.ts.insert(
        path.clone(),
        CheckFileFacts {
            parse_error: Some("primary parse error".to_string()),
            ..CheckFileFacts::default()
        },
    );
    let fallback = TsFactMap::from([(
        path.clone(),
        TsFileFacts {
            parse_error: Some("fallback parse error".to_string()),
            ..TsFileFacts::default()
        },
    )]);
    let lookup = FallbackTsFactLookup::new(
        &primary,
        &fallback,
        false,
        std::slice::from_ref(&path),
        &HashSet::from([path.clone()]),
    );

    let result = lookup
        .get_playwright_fetch_facts(&path)
        .expect("matching graph universe reuses per-file fetch facts");
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("primary parse error must be retained"),
    };
    assert!(error.contains("primary parse error"));
    assert!(!error.contains("fallback parse error"));
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
        .get_or_compute_app_selector_occurrences(&cache_settings(), false, &|| {
            primary_selector_calls.set(primary_selector_calls.get() + 1);
            Ok(Vec::new())
        })
        .expect("primary selectors compute");
    let primary_route_calls = Cell::new(0);
    primary.get_or_compute_playwright_routes(&cache_settings(), &|| {
        primary_route_calls.set(primary_route_calls.get() + 1);
        Vec::new()
    });
    let primary_text_calls = Cell::new(0);
    primary
        .get_or_compute_app_text_targets(&cache_settings(), &|| {
            primary_text_calls.set(primary_text_calls.get() + 1);
            Ok(Vec::new())
        })
        .expect("primary text targets compute");
    let primary_reachability_calls = Cell::new(0);
    primary
        .get_or_compute_route_reachable_files(&cache_settings(), &|| {
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
    assert!(lookup.playwright_source_files().is_none());
    assert!(lookup.get_playwright_test_files(None).is_none());
    assert!(lookup.get_playwright_fetch_facts(&graph_path).is_none());

    let selector_calls = Cell::new(0);
    for _ in 0..2 {
        lookup
            .get_or_compute_app_selector_occurrences(&cache_settings(), false, &|| {
                selector_calls.set(selector_calls.get() + 1);
                Ok(Vec::new())
            })
            .expect("isolated selectors compute");
    }
    let route_calls = Cell::new(0);
    for _ in 0..2 {
        lookup.get_or_compute_playwright_routes(&cache_settings(), &|| {
            route_calls.set(route_calls.get() + 1);
            Vec::new()
        });
    }
    let text_calls = Cell::new(0);
    let reachability_calls = Cell::new(0);
    for _ in 0..2 {
        lookup
            .get_or_compute_app_text_targets(&cache_settings(), &|| {
                text_calls.set(text_calls.get() + 1);
                Ok(Vec::new())
            })
            .expect("isolated text targets compute");
        lookup
            .get_or_compute_route_reachable_files(&cache_settings(), &|| {
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
    )
    .expect("route-import graph builds");
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
