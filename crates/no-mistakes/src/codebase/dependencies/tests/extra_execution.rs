#[test]
fn run_covers_lazy_import_normal_graph_filters_formats_and_timings() {
    let root = simple_root();

    let mut lazy = traverse_args(root.clone(), vec![PathBuf::from("a.mts")]);
    lazy.relationships = vec![RelationshipArg::Import];
    lazy.format = Some(Format::Md);
    lazy.timings = true;
    run(lazy, Direction::Deps).unwrap();

    let mut normal = traverse_args(root.clone(), vec![PathBuf::from("a.mts")]);
    normal.relationships = vec![RelationshipArg::All];
    normal.filters = vec!["*.mts".to_string()];
    normal.tests = vec!["vitest".to_string()];
    normal.format = Some(Format::Yml);
    run(normal, Direction::Deps).unwrap();

    let mut paths = traverse_args(root, vec![PathBuf::from("a.mts")]);
    paths.format = Some(Format::Paths);
    run(paths, Direction::Deps).unwrap();
}

#[test]
fn run_with_cwd_and_writer_surfaces_output_errors() {
    let root = simple_root();
    let args = traverse_args(root, vec![PathBuf::from("a.mts")]);
    let cwd = std::env::current_dir().unwrap();
    let mut out = FailingWriter;
    let mut timings = crate::codebase::timing::PhaseTimings::start();

    let result = collect_and_filter_entries(&args, Direction::Deps, &cwd, &mut timings).unwrap();
    let root_strs: Vec<String> = args.files.iter().map(|f| f.display().to_string()).collect();
    let err = write_output_results(Format::Json, &root_strs, &result, &mut out).unwrap_err();
    timings.mark("output");

    assert!(err.to_string().contains("synthetic write failure"));
    assert!(timings
        .phases
        .iter()
        .any(|(label, _duration)| *label == "output"));
}

#[test]
fn run_dependents_covers_mixed_symbol_and_plain_entrypoints() {
    let root = symbol_root();
    let mut args = traverse_args(
        root,
        vec![
            PathBuf::from("source.mts#alpha"),
            PathBuf::from("uses-alpha.mts"),
        ],
    );
    args.relationships = vec![RelationshipArg::Import];
    args.format = Some(Format::Human);

    run(args, Direction::Dependents).unwrap();
}

#[test]
fn shared_traversal_rebuilds_without_symbols_for_plain_reports() {
    let root = symbol_root();
    let cwd = std::env::current_dir().unwrap();
    let mut shared = SharedTraversalContext::prepare(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan::all().with_symbols(true),
    )
    .unwrap();

    let mut deps = traverse_args(root.clone(), vec![PathBuf::from("source.mts")]);
    deps.relationships = vec![RelationshipArg::All];
    collect_and_filter_entries_shared(&deps, Direction::Deps, &cwd, &mut shared).unwrap();

    let mut dependents = traverse_args(root, vec![PathBuf::from("source.mts")]);
    dependents.relationships = vec![RelationshipArg::All];
    collect_and_filter_entries_shared(&dependents, Direction::Dependents, &cwd, &mut shared)
        .unwrap();

    assert_eq!(shared.graph_builds, 1);
}

#[test]
fn shared_traversal_symbol_dependents_use_symbol_free_import_graph_when_preplanned() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("tests-impact-symbol")
        .join("fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let cwd = std::env::current_dir().unwrap();
    let mut shared = SharedTraversalContext::prepare(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan::all().with_symbols(true),
    )
    .unwrap();

    let mut args = traverse_args(root.clone(), vec![PathBuf::from("utils.mts#parseDate")]);
    args.relationships = vec![RelationshipArg::Import];
    let result =
        collect_and_filter_entries_shared(&args, Direction::Dependents, &cwd, &mut shared)
            .unwrap();

    assert_eq!(shared.graph_builds, 1);
    assert_eq!(result.root, root);
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
    assert_eq!(
        check_facts.ts[&unit].ts.imports.len(),
        shared.facts.as_ref().unwrap()[&unit].imports.len(),
    );

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
fn traversal_stages_graph_configuration_around_one_prepared_test_project_pass() {
    let shared = include_str!("../shared_traversal.rs");
    let shared_graph = concat!(
        include_str!("../shared_traversal_graph.rs"),
        include_str!("../shared_graph_cache.rs"),
    );
    let standalone = include_str!("../mod.rs");

    assert_eq!(shared.matches("AnalysisDataset::new(&root)").count(), 1);
    assert_eq!(shared.matches("dataset.config(config_path)?").count(), 1);
    assert_eq!(shared.matches("dataset.tsconfig(tsconfig_path)?").count(), 1);
    assert_eq!(shared.matches("config_from_loaded_v2(").count(), 1);
    assert_eq!(
        shared
            .matches("prepare_graph_config_with_test_filter_and_workspace(")
            .count(),
        2
    );
    assert_eq!(shared.matches("TestFileFilter::fallback_only()").count(), 1);
    assert_eq!(
        shared
            .matches("prepare_test_projects_from_visible_with_sources(")
            .count(),
        1
    );
    assert_eq!(shared.matches("TestFileFilter::from_prepared_projects(").count(), 1);
    assert_eq!(
        shared
            .matches("ts_fact_plan_and_context_for_plan_with_prepared(")
            .count(),
        2
    );
    assert_eq!(
        shared_graph
            .matches("build_with_plan_files_prepared_config_facts_and_resolution_cache(")
            .count(),
        1
    );
    assert!(!shared.contains("graph_config_options"));
    assert!(!shared.contains("load_v2_config("));

    let test_filter = standalone
        .split("fn test_filters_from_prepared(")
        .nth(1)
        .and_then(|source| source.split("fn apply_target_module_filters(").next())
        .expect("prepared standalone test-filter helper");
    assert!(test_filter.contains("discover_tests_from_prepared_projects"));
    assert!(test_filter.contains("discover_tests_from_visible"));
    assert!(!test_filter.contains("load_v2_config"));
    assert!(!test_filter.contains("discover_visible_paths"));
}

#[test]
fn shared_traversal_reuses_runner_helpers_for_lazy_symbols_and_test_filters() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/shared-traversal-prepared-projects"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let cwd = std::env::current_dir().unwrap();
    crate::ast::begin_parse_count(&root);
    let mut shared = SharedTraversalContext::prepare(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan {
            imports: true,
            symbols: true,
            ..Default::default()
        },
    )
    .unwrap();

    let mut lazy = traverse_args(root.clone(), vec![PathBuf::from("src/unit.test.ts")]);
    lazy.relationships = vec![RelationshipArg::Import];
    lazy.tests = vec!["vitest".to_string()];
    let lazy_result =
        collect_and_filter_entries_shared(&lazy, Direction::Deps, &cwd, &mut shared).unwrap();
    assert!(lazy_result.entries.is_empty());

    let mut symbol = traverse_args(root.clone(), vec![PathBuf::from("src/unit.ts#unit")]);
    symbol.relationships = vec![RelationshipArg::Import];
    symbol.tests = vec!["vitest".to_string()];
    let _symbol_result =
        collect_and_filter_entries_shared(&symbol, Direction::Dependents, &cwd, &mut shared)
            .unwrap();

    let mut excluded =
        traverse_args(root.clone(), vec![PathBuf::from("src/excluded.ts#excluded")]);
    excluded.relationships = vec![RelationshipArg::Import];
    excluded.tests = vec!["vitest".to_string()];
    let excluded_result =
        collect_and_filter_entries_shared(&excluded, Direction::Dependents, &cwd, &mut shared)
            .unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert!(excluded_result.entries.is_empty());
    assert_eq!(shared.graph_builds, 1);
    assert_eq!(shared.symbol_index_builds, 1);
    let classifications = shared.import_resolution_cache.classification_count();
    assert!(classifications > 0);
    assert!(
        shared.import_resolution_cache.request_count() > classifications,
        "graph consumers should reuse request-scoped import classifications"
    );
    assert_eq!(counts.len(), 6, "{counts:#?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
}

#[test]
fn shared_import_only_traversal_parses_only_reachable_files() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/lazy-import/fixture"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let cwd = std::env::current_dir().unwrap();
    crate::ast::begin_parse_count(&root);
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
    // Exercise the first lazy consumer: it must create a sparse fact map, not force a full one.
    shared.facts = None;
    let mut args = traverse_args(root.clone(), vec![PathBuf::from("src/a.mts")]);
    args.relationships = vec![RelationshipArg::Import];

    let result =
        collect_and_filter_entries_shared(&args, Direction::Deps, &cwd, &mut shared).unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(
        result
            .entries
            .iter()
            .filter_map(|entry| entry.node.as_file())
            .collect::<Vec<_>>(),
        vec![root.join("src/b.mts").as_path()]
    );
    assert_eq!(counts.get(&root.join("src/a.mts")), Some(&1), "{counts:#?}");
    assert_eq!(counts.get(&root.join("src/b.mts")), Some(&1), "{counts:#?}");
    // The lazy reader parses immediately after a successful read, so the absence of parse
    // records also proves that the unreachable component was not read for source analysis.
    assert!(!counts.contains_key(&root.join("src/unrelated.mts")), "{counts:#?}");
    assert!(
        !counts.contains_key(&root.join("src/unrelated-dep.mts")),
        "{counts:#?}"
    );
    assert_eq!(counts.len(), 2, "{counts:#?}");
    assert_eq!(shared.facts.as_ref().map(|facts| facts.len()), Some(2));
    assert_eq!(shared.graph_builds, 0);
    assert_eq!(shared.import_resolution_cache.classification_count(), 1);
    assert_eq!(shared.import_resolution_cache.request_count(), 1);
}

#[test]
fn shared_lazy_import_facts_are_reused_by_later_symbol_queries() {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/lazy-import/fixture"),
    );
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let cwd = std::env::current_dir().unwrap();
    crate::ast::begin_parse_count(&root);
    let mut shared = SharedTraversalContext::prepare(
        root.clone(),
        None,
        None,
        graph::GraphBuildPlan {
            imports: true,
            symbols: true,
            ..Default::default()
        },
    )
    .unwrap();
    let mut args = traverse_args(root.clone(), vec![PathBuf::from("src/a.mts")]);
    args.relationships = vec![RelationshipArg::Import];

    collect_and_filter_entries_shared(&args, Direction::Deps, &cwd, &mut shared).unwrap();

    let lazy_facts = shared.facts.as_ref().expect("lazy facts are retained");
    assert!(lazy_facts.contains_key(&root.join("src/a.mts")));
    assert!(lazy_facts.contains_key(&root.join("src/b.mts")));
    assert!(!lazy_facts.contains_key(&root.join("src/unrelated.mts")));
    assert!(!lazy_facts.contains_key(&root.join("src/unrelated-dep.mts")));

    // A later symbol/full-graph consumer fills only the remaining fact entries.
    assert_eq!(shared.facts().len(), 4);
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.len(), 4, "{counts:#?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
    assert_eq!(shared.graph_builds, 0);
}

#[test]
fn traversal_queue_root_helpers_cover_missing_deps_and_module_entrypoints() {
    let file = PathBuf::from("/repo/src/queue.ts");
    let roots = vec![
        NodeId::File(file.clone()),
        NodeId::Module("queue-package".to_string()),
    ];
    let expanded = roots_with_exported_symbol_roots_by(&roots, |_| None);
    assert_eq!(expanded, roots);

    let entrypoints = vec![Entrypoint {
        file,
        node: NodeId::Module("queue-package".to_string()),
        symbol: Some("send".to_string()),
    }];
    let queue_roots = roots_with_existing_queue_jobs_by(&expanded, &entrypoints, |_| true);
    assert_eq!(queue_roots, expanded);
}

#[test]
fn dependents_treats_module_symbol_entrypoints_as_module_roots() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("graph-modules")
        .join("fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let mut args = traverse_args(root.clone(), vec![PathBuf::from("@react/client#handler")]);
    args.relationships = vec![RelationshipArg::Import];
    let cwd = std::env::current_dir().unwrap();
    let mut timings = crate::codebase::timing::PhaseTimings::start();

    let result =
        collect_and_filter_entries(&args, Direction::Dependents, &cwd, &mut timings).unwrap();

    assert!(result
        .entries
        .iter()
        .any(|entry| entry.node.as_file() == Some(root.join("src/entry.mts").as_path())));
}

#[test]
fn dependents_finds_tsconfig_alias_importers() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("dependents-tsconfig-alias")
        .join("fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let mut args = traverse_args(root.clone(), vec![PathBuf::from("components/button.tsx")]);
    args.relationships = vec![RelationshipArg::Import];
    let cwd = std::env::current_dir().unwrap();
    let mut timings = crate::codebase::timing::PhaseTimings::start();

    let result =
        collect_and_filter_entries(&args, Direction::Dependents, &cwd, &mut timings).unwrap();

    let files: Vec<_> = result
        .entries
        .iter()
        .filter_map(|e| e.node.as_file().map(|p| p.to_path_buf()))
        .collect();
    assert!(
        files.iter().any(|f| f == &root.join("pages/home.tsx")),
        "should find pages/home.tsx (imports via @/ alias), got: {files:?}"
    );
    assert!(
        files.iter().any(|f| f == &root.join("pages/settings.tsx")),
        "should find pages/settings.tsx (imports via @/ alias)"
    );
    assert!(
        files
            .iter()
            .any(|f| f == &root.join("tests/button.test.tsx")),
        "should find tests/button.test.tsx (direct relative import)"
    );
}

#[test]
fn relationship_arg_as_str_all_variants() {
    assert_eq!(RelationshipArg::Import.as_str(), "import");
    assert_eq!(RelationshipArg::ImportStatic.as_str(), "import-static");
    assert_eq!(RelationshipArg::ImportDynamic.as_str(), "import-dynamic");
    assert_eq!(RelationshipArg::ImportType.as_str(), "import-type");
    assert_eq!(RelationshipArg::ImportRequire.as_str(), "import-require");
    assert_eq!(RelationshipArg::RouteImport.as_str(), "route-import");
    assert_eq!(RelationshipArg::Workspace.as_str(), "workspace");
    assert_eq!(RelationshipArg::Package.as_str(), "package");
    assert_eq!(RelationshipArg::Test.as_str(), "test");
    assert_eq!(RelationshipArg::Route.as_str(), "route");
    assert_eq!(RelationshipArg::Queue.as_str(), "queue");
    assert_eq!(RelationshipArg::Md.as_str(), "md");
    assert_eq!(RelationshipArg::Ci.as_str(), "ci");
    assert_eq!(RelationshipArg::Http.as_str(), "http");
    assert_eq!(RelationshipArg::Process.as_str(), "process");
    assert_eq!(RelationshipArg::Asset.as_str(), "asset");
    assert_eq!(RelationshipArg::React.as_str(), "react");
    assert_eq!(RelationshipArg::Dotnet.as_str(), "dotnet");
    assert_eq!(RelationshipArg::Swift.as_str(), "swift");
    assert_eq!(RelationshipArg::Terraform.as_str(), "terraform");
    assert_eq!(RelationshipArg::All.as_str(), "all");
}

#[test]
fn terraform_relationship_maps_to_terraform_edge_kinds() {
    let filter = relationship_filter(&[RelationshipArg::Terraform]).expect("filter set");
    assert!(filter.contains(&EdgeKind::TerraformReference));
    assert!(filter.contains(&EdgeKind::TerraformModuleRef));
    assert!(filter.contains(&EdgeKind::TerraformOutputRef));
}
