use super::super::*;

#[test]
fn http_and_process_relationships_map_to_edge_kinds() {
    let set = relationship_filter(&[
        RelationshipArg::Http,
        RelationshipArg::Process,
        RelationshipArg::Asset,
        RelationshipArg::React,
    ])
    .unwrap();
    assert!(set.contains(&EdgeKind::HttpCall));
    assert!(set.contains(&EdgeKind::ProcessSpawn));
    assert!(set.contains(&EdgeKind::AssetImport));
    assert!(set.contains(&EdgeKind::ReactRender));
    assert!(relationship_filter(&[RelationshipArg::All]).is_none());
    assert!(relationship_filter(&[]).is_none());
}

#[test]
fn import_only_detection_requires_nonempty_all_import_relationships() {
    assert!(!relationships_are_import_only(&[]));
    assert!(relationships_are_import_only(&[RelationshipArg::Import]));
    assert!(relationships_are_import_only(&[
        RelationshipArg::ImportStatic
    ]));
    assert!(relationships_are_import_only(&[
        RelationshipArg::ImportDynamic
    ]));
    assert!(relationships_are_import_only(&[
        RelationshipArg::ImportType
    ]));
    assert!(relationships_are_import_only(&[
        RelationshipArg::ImportRequire
    ]));
    assert!(relationships_are_import_only(&[
        RelationshipArg::ImportStatic,
        RelationshipArg::ImportDynamic,
        RelationshipArg::ImportType,
        RelationshipArg::ImportRequire,
    ]));
    assert!(!relationships_are_import_only(&[
        RelationshipArg::ImportStatic,
        RelationshipArg::Test,
    ]));
}

#[test]
fn resolve_format_prefers_flags_then_tty_default() {
    assert_eq!(
        resolve_format(true, Some(Format::Human), true),
        Format::Json
    );
    assert_eq!(resolve_format(false, Some(Format::Md), true), Format::Md);
    assert_eq!(resolve_format(false, None, true), Format::Human);
    assert_eq!(resolve_format(false, None, false), Format::Json);
}

#[test]
fn merge_node_entries_keeps_min_depth_and_dedupes_edge_kinds() {
    let node = NodeId::File(PathBuf::from("shared.ts"));
    let mut merged = HashMap::new();
    merge_node_entries(
        &mut merged,
        vec![graph::NodeEntry {
            node: node.clone(),
            depth: 3,
            via: vec![EdgeKind::Import],
        }],
    );
    merge_node_entries(
        &mut merged,
        vec![graph::NodeEntry {
            node: node.clone(),
            depth: 1,
            via: vec![EdgeKind::Import, EdgeKind::TestOf],
        }],
    );

    let entry = merged.get(&node).unwrap();
    assert_eq!(entry.depth, 1);
    assert_eq!(entry.via, vec![EdgeKind::Import, EdgeKind::TestOf]);
}

#[test]
fn symbol_roots_keep_matching_queue_job_roots() {
    let queue_file = PathBuf::from("/repo/src/queues.ts");
    let symbol_root = NodeId::Symbol {
        file: queue_file.clone(),
        symbol: "sendWelcome".to_string(),
    };
    let queue_job = NodeId::QueueJob {
        queue_file: queue_file.clone(),
        job: "sendWelcome".to_string(),
    };
    let entrypoints = vec![Entrypoint {
        file: queue_file,
        node: symbol_root.clone(),
        symbol: Some("sendWelcome".to_string()),
    }];

    let roots =
        roots_with_existing_queue_jobs_by(&[symbol_root], &entrypoints, |node| node == &queue_job);

    assert!(roots.contains(&queue_job));
}

#[test]
fn target_module_filter_keeps_only_matching_module_nodes() {
    let entries = vec![
        graph::NodeEntry {
            node: NodeId::Module("@react/client".to_string()),
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        graph::NodeEntry {
            node: NodeId::Module("lodash".to_string()),
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        graph::NodeEntry {
            node: NodeId::File(PathBuf::from("src/local.mts")),
            depth: 1,
            via: vec![EdgeKind::Import],
        },
    ];

    let filtered = apply_target_module_filters(entries, &["@react/*".to_string()]).unwrap();

    assert_eq!(filtered.len(), 1);
    assert_eq!(
        filtered[0].node,
        NodeId::Module("@react/client".to_string())
    );
}

#[test]
fn file_filters_exclude_module_nodes_without_target_module_filter() {
    let entries = node_entries_fixture("module-queue-file-filter.json");
    let mut args = traverse_args(PathBuf::from("/repo"), vec![PathBuf::from("src/entry.mts")]);
    args.filters = vec!["src/**".to_string()];

    let filtered = apply_filters(entries, &args, Path::new("/repo")).unwrap();

    assert_eq!(filtered.len(), 2);
    assert!(filtered
        .iter()
        .any(|entry| entry.node == NodeId::File(PathBuf::from("/repo/src/local.mts"))));
    assert!(filtered
        .iter()
        .any(|entry| matches!(entry.node, NodeId::QueueJob { .. })));
}

fn node_entries_fixture(name: &str) -> Vec<graph::NodeEntry> {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/node-entries/fixture")
        .join(name);
    let source = std::fs::read_to_string(&fixture).expect("fixture file should exist");
    let values: Vec<serde_json::Value> = serde_json::from_str(&source).unwrap();
    values.into_iter().map(node_entry_from_json).collect()
}

fn node_entry_from_json(value: serde_json::Value) -> graph::NodeEntry {
    let node = value.get("node").unwrap();
    let node = if let Some(module) = node.get("module").and_then(|value| value.as_str()) {
        NodeId::Module(module.to_string())
    } else if let Some(file) = node.get("file").and_then(|value| value.as_str()) {
        NodeId::File(PathBuf::from(file))
    } else {
        NodeId::QueueJob {
            queue_file: PathBuf::from(node["queue_file"].as_str().unwrap()),
            job: node["job"].as_str().unwrap().to_string(),
        }
    };
    let via = value["via"]
        .as_array()
        .unwrap()
        .iter()
        .map(|kind| match kind.as_str().unwrap() {
            "import" => EdgeKind::Import,
            "queue-enqueue" => EdgeKind::QueueEnqueue,
            other => panic!("unsupported fixture edge kind {other}"),
        })
        .collect();
    graph::NodeEntry {
        node,
        depth: value["depth"].as_u64().unwrap() as usize,
        via,
    }
}

#[test]
fn deps_direction_rejects_symbol_entrypoints() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("simple")
        .join("fixture");
    let args = TraverseArgs {
        files: vec![PathBuf::from("a.mts#a")],
        file_symbols: Vec::new(),
        root: Some(root),
        tsconfig: None,
        depth: None,
        filters: Vec::new(),
        target_modules: Vec::new(),
        tests: Vec::new(),
        relationships: Vec::new(),
        include_symbols: false,
        format: Some(Format::Json),
        json: false,
        timings: false,
    };

    let err = run(args, Direction::Deps).unwrap_err();

    assert!(err.to_string().contains("#symbol targeting"));
}

struct FailingWriter;

impl std::io::Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("synthetic write failure"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn simple_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("simple")
        .join("fixture")
}

fn symbol_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("symbol-export")
        .join("fixture")
}

fn traverse_args(root: PathBuf, files: Vec<PathBuf>) -> TraverseArgs {
    TraverseArgs {
        files,
        file_symbols: Vec::new(),
        root: Some(root),
        tsconfig: None,
        depth: Some(3),
        filters: Vec::new(),
        target_modules: Vec::new(),
        tests: Vec::new(),
        relationships: Vec::new(),
        include_symbols: false,
        format: Some(Format::Json),
        json: false,
        timings: false,
    }
}

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
    let tsconfig = resolve_tsconfig(&traverse_args(root.clone(), Vec::new()), &root).unwrap();
    let graph_files = graph::GraphFiles::discover(&root);
    let cwd = std::env::current_dir().unwrap();
    let mut shared = SharedTraversalContext::new(root.clone(), tsconfig, graph_files);
    shared.include_plan(graph::GraphBuildPlan::all().with_symbols(true));

    let mut deps = traverse_args(root.clone(), vec![PathBuf::from("source.mts")]);
    deps.relationships = vec![RelationshipArg::Import];
    collect_and_filter_entries_shared(&deps, Direction::Deps, &cwd, &mut shared).unwrap();

    let mut dependents = traverse_args(root, vec![PathBuf::from("source.mts")]);
    dependents.relationships = vec![RelationshipArg::Import];
    collect_and_filter_entries_shared(&dependents, Direction::Dependents, &cwd, &mut shared)
        .unwrap();

    assert_eq!(shared.graph_builds, 0);
}

#[test]
fn shared_traversal_symbol_dependents_use_symbol_free_import_graph_when_preplanned() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("tests-impact-symbol")
        .join("fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let tsconfig = resolve_tsconfig(&traverse_args(root.clone(), Vec::new()), &root).unwrap();
    let graph_files = graph::GraphFiles::discover(&root);
    let cwd = std::env::current_dir().unwrap();
    let mut shared = SharedTraversalContext::new(root.clone(), tsconfig, graph_files);
    shared.include_plan(graph::GraphBuildPlan::all().with_symbols(true));

    let mut args = traverse_args(root.clone(), vec![PathBuf::from("utils.mts#parseDate")]);
    args.relationships = vec![RelationshipArg::Import];
    let result =
        collect_and_filter_entries_shared(&args, Direction::Dependents, &cwd, &mut shared)
            .unwrap();

    assert_eq!(shared.graph_builds, 0);
    assert_eq!(result.root, root);
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
