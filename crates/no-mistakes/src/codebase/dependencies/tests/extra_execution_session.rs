#[test]
fn shared_traversal_reuses_equivalent_symbol_free_graphs_for_plain_reports() {
    let root = symbol_root();
    let cwd = std::env::current_dir().unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let mut shared = SharedTraversalContext::prepare_with_session(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan::all().with_symbols(true),
        session,
    )
    .unwrap();

    let mut deps = traverse_args(root.clone(), vec![PathBuf::from("source.mts")]);
    deps.relationships = vec![RelationshipArg::All];
    collect_and_filter_entries_shared(&deps, Direction::Deps, &cwd, &mut shared).unwrap();

    let mut dependents = traverse_args(root, vec![PathBuf::from("source.mts")]);
    dependents.relationships = vec![RelationshipArg::All];
    collect_and_filter_entries_shared(&dependents, Direction::Dependents, &cwd, &mut shared)
        .unwrap();

    let work = observer.snapshot().work;
    assert_eq!(shared.graph_builds, 0);
    assert_eq!(work["graph.builds"], 1);
    assert_eq!(work["graph.reuses"], 1);
    assert_eq!(work["traversal.computations"], 2);
}

#[test]
fn shared_traversal_symbol_dependents_use_symbol_free_import_graph_when_preplanned() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("tests-impact-symbol")
        .join("fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let cwd = std::env::current_dir().unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let mut shared = SharedTraversalContext::prepare_with_session(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan::all().with_symbols(true),
        session,
    )
    .unwrap();

    let mut args = traverse_args(root.clone(), vec![PathBuf::from("utils.mts#parseDate")]);
    args.relationships = vec![RelationshipArg::Import];
    let result =
        collect_and_filter_entries_shared(&args, Direction::Dependents, &cwd, &mut shared)
            .unwrap();

    let mut second = traverse_args(root.clone(), vec![PathBuf::from("utils.mts#formatDate")]);
    second.relationships = vec![RelationshipArg::Import];
    collect_and_filter_entries_shared(&second, Direction::Dependents, &cwd, &mut shared).unwrap();

    let work = observer.snapshot().work;
    assert_eq!(shared.graph_builds, 0);
    assert_eq!(result.root, root);
    assert_eq!(work["graph.builds"], 1);
    assert_eq!(work["graph.reuses"], 1);
    assert_eq!(work["symbol_index.builds"], 1);
    assert_eq!(work["symbol_index.reuses"], 1);
}

#[test]
fn shared_traversal_reuses_equivalent_traversal_results_before_filtering() {
    let root = simple_root();
    let cwd = std::env::current_dir().unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let mut shared = SharedTraversalContext::prepare_with_session(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan::all(),
        session,
    )
    .unwrap();
    let args = traverse_args(root.clone(), vec![PathBuf::from("a.mts")]);
    let mut filtered = traverse_args(root, vec![PathBuf::from("a.mts")]);
    filtered.filters = vec!["does-not-exist/**".to_string()];

    let first =
        collect_and_filter_entries_shared(&args, Direction::Deps, &cwd, &mut shared).unwrap();
    let second =
        collect_and_filter_entries_shared(&filtered, Direction::Deps, &cwd, &mut shared).unwrap();

    let work = observer.snapshot().work;
    assert!(!first.entries.is_empty());
    assert!(second.entries.is_empty());
    assert_eq!(work["graph.builds"], 1);
    assert_eq!(work["traversal.requests"], 2);
    assert_eq!(work["traversal.computations"], 1);
    assert_eq!(work["traversal.reuses"], 1);
}

#[test]
fn shared_traversal_initializes_absent_fact_maps_for_empty_and_nonempty_universes() {
    let root = simple_root();
    let mut shared = SharedTraversalContext::prepare(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan {
            imports: true,
            ..Default::default()
        },
    )
    .unwrap();

    // These resets protect the defensive lazy initialization used when a
    // caller replaces or clears precollected facts before requesting a graph.
    shared.graph_files = graph::GraphFiles::from_files(Vec::new());
    shared.facts = None;
    assert!(shared.facts().is_empty());

    let source = root.join("a.mts");
    shared.graph_files = graph::GraphFiles::from_files(vec![source.clone()]);
    shared
        .fact_context
        .set_visible_files(std::iter::once(source.clone()));
    shared.facts = None;
    assert!(shared.facts().contains_key(&source));

    shared.graph = None;
    shared.graph().expect("graph builds from newly collected facts");
    shared.graph().expect("graph is reused after the first build");
    assert_eq!(shared.graph_builds, 1);
}

#[test]
fn traversal_work_metrics_distinguish_lazy_and_canonical_graph_paths() {
    let root = simple_root();
    let cwd = std::env::current_dir().unwrap();

    let lazy_observer = crate::diagnostics::InvocationObserver::new(true);
    let lazy_session = crate::codebase::analysis_session::AnalysisSession::new(Some(
        lazy_observer.clone(),
    ));
    let mut lazy_shared = SharedTraversalContext::prepare_with_session(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan {
            imports: true,
            ..Default::default()
        },
        lazy_session,
    )
    .unwrap();
    let mut lazy = traverse_args(root.clone(), vec![PathBuf::from("a.mts")]);
    lazy.relationships = vec![RelationshipArg::Import];
    collect_and_filter_entries_shared(&lazy, Direction::Deps, &cwd, &mut lazy_shared).unwrap();
    assert_eq!(
        lazy_observer
            .snapshot()
            .work
            .get("graph.builds")
            .copied()
            .unwrap_or_default(),
        0
    );

    let graph_observer = crate::diagnostics::InvocationObserver::new(true);
    let graph_session = crate::codebase::analysis_session::AnalysisSession::new(Some(
        graph_observer.clone(),
    ));
    let mut graph_shared = SharedTraversalContext::prepare_with_session(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan::all().with_symbols(false),
        graph_session,
    )
    .unwrap();
    let mut normal = traverse_args(root, vec![PathBuf::from("a.mts")]);
    normal.relationships = vec![RelationshipArg::All];
    collect_and_filter_entries_shared(&normal, Direction::Deps, &cwd, &mut graph_shared).unwrap();
    assert_eq!(graph_observer.snapshot().work["graph.builds"], 1);
    assert_eq!(graph_shared.graph_builds, 1);
}

#[test]
fn shared_traversal_extends_absent_facts_and_seeds_cached_program_facts() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/shared-traversal-prepared-projects"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let unit = root.join("src/unit.ts");
    let excluded = root.join("src/excluded.ts");
    let missing = root.join("src/missing.ts");
    let mut shared = SharedTraversalContext::prepare(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan {
            imports: true,
            ..Default::default()
        },
    )
    .unwrap();
    let check_facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        vec![unit.clone()],
        crate::codebase::check_facts::CheckFactPlan {
            graph: crate::codebase::ts_source::facts::TsFactPlan::imports(),
            ..Default::default()
        },
    );

    shared.facts = None;
    shared.extend_check_facts(&check_facts);
    assert!(shared.facts.as_ref().unwrap().contains_key(&unit));
    assert!(!shared.facts.as_ref().unwrap().contains_key(&excluded));

    shared.seed_cached_program_facts(&std::collections::HashSet::from([
        unit.clone(),
        excluded.clone(),
        missing.clone(),
    ]));
    let facts = shared.facts.as_ref().unwrap();
    assert!(facts.contains_key(&unit));
    assert!(facts.contains_key(&excluded));
    assert!(!facts.contains_key(&missing));

    shared.facts = None;
    shared.seed_cached_program_facts(&std::collections::HashSet::from([unit.clone()]));
    assert!(shared.facts.as_ref().unwrap().contains_key(&unit));
}

#[test]
fn shared_import_traversal_only_reads_and_parses_reachable_frontier() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/lazy-import/fixture"),
    );
    let entry = root.join("src/a.mts");
    let reached = root.join("src/b.mts");
    let unrelated = root.join("src/unrelated.mts");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(
        std::sync::Arc::clone(&observer),
    ));
    let mut shared = SharedTraversalContext::prepare_with_session(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan {
            imports: true,
            ..Default::default()
        },
        std::sync::Arc::clone(&session),
    )
    .unwrap();
    let mut args = traverse_args(root.clone(), vec![PathBuf::from("src/a.mts")]);
    args.relationships = vec![RelationshipArg::Import];

    let result = crate::ast::with_request_parse_cache(|| {
        collect_and_filter_entries_shared(&args, Direction::Deps, &root, &mut shared).unwrap()
    });

    assert_eq!(
        result
            .entries
            .iter()
            .filter_map(|entry| entry.node.as_file())
            .collect::<Vec<_>>(),
        vec![reached.as_path()]
    );
    let work = session.work_snapshot();
    assert_eq!(work.source_reads.len(), 2, "{:?}", work.source_reads);
    assert_eq!(work.source_reads[&entry], 1);
    assert_eq!(work.source_reads[&reached], 1);
    assert!(!work.source_reads.contains_key(&unrelated));
    assert_eq!(work.parse_attempts.len(), 2, "{:?}", work.parse_attempts);
    assert_eq!(work.parse_attempts[&entry], 1);
    assert_eq!(work.parse_attempts[&reached], 1);
    assert!(!work.parse_attempts.contains_key(&unrelated));
}

#[test]
fn shared_traversal_reuses_workspace_manifest_documents_across_lazy_and_symbol_paths() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/performance/core-analysis"),
    );
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(
        std::sync::Arc::clone(&observer),
    ));
    let mut shared = SharedTraversalContext::prepare_with_session(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan {
            imports: true,
            symbols: true,
            ..Default::default()
        },
        std::sync::Arc::clone(&session),
    )
    .unwrap();
    let baseline = observer
        .snapshot()
        .work
        .get("manifest.parses")
        .copied()
        .unwrap_or(0);

    crate::ast::with_request_parse_cache(|| {
        let mut lazy = traverse_args(root.clone(), vec![PathBuf::from("src/app.tsx")]);
        lazy.relationships = vec![RelationshipArg::Import];
        lazy.depth = Some(1);
        collect_and_filter_entries_shared(&lazy, Direction::Deps, &root, &mut shared).unwrap();
        let after_first = observer.snapshot().work["manifest.parses"];
        assert!(after_first > baseline);

        lazy.depth = Some(2);
        collect_and_filter_entries_shared(&lazy, Direction::Deps, &root, &mut shared).unwrap();
        let mut symbol =
            traverse_args(root.clone(), vec![PathBuf::from("src/app.tsx#App")]);
        symbol.relationships = vec![RelationshipArg::Import];
        collect_and_filter_entries_shared(&symbol, Direction::Dependents, &root, &mut shared)
            .unwrap();

        let work = observer.snapshot().work;
        assert_eq!(work["manifest.parses"], after_first, "{work:#?}");
        assert!(work["manifest.cache_hits"] >= 6, "{work:#?}");
    });
}

#[test]
fn shared_traversal_seed_uses_invocation_source_and_parser_gateways_once() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/shared-traversal-prepared-projects"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let excluded = root.join("src/excluded.ts");
    let missing = root.join("src/missing.ts");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(
        std::sync::Arc::clone(&observer),
    ));

    crate::ast::with_request_parse_cache(|| {
        let mut shared = SharedTraversalContext::prepare_with_session(
            root,
            None,
            None,
            graph::GraphBuildPlan {
                imports: true,
                ..Default::default()
            },
            std::sync::Arc::clone(&session),
        )
        .unwrap();
        let requested = std::collections::HashSet::from([excluded.clone(), missing.clone()]);
        for _ in 0..2 {
            shared.facts = None;
            shared.seed_cached_program_facts(&requested);
            assert!(shared.facts.as_ref().unwrap().contains_key(&excluded));
            assert!(!shared.facts.as_ref().unwrap().contains_key(&missing));
        }
    });

    let work = session.work_snapshot();
    assert_eq!(work.source_reads[&excluded], 1);
    assert_eq!(work.source_reads[&missing], 1);
    assert_eq!(work.parse_attempts[&excluded], 1);
    assert!(!work.parse_attempts.contains_key(&missing));
    let metrics = observer.snapshot().work;
    assert_eq!(metrics["source.requests"], 4);
    assert_eq!(metrics["source.reads"], 2);
    assert_eq!(metrics["source.cache_hits"], 2);
    assert_eq!(metrics["source.read_errors"], 1);
    assert_eq!(metrics["parse.requests"], 2);
    assert_eq!(metrics["parse.files"], 1);
}
