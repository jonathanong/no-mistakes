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
    assert!(!relationship_filter(&[RelationshipArg::All])
        .expect("all uses standard edges")
        .contains(&EdgeKind::RouteImport));
    assert!(!relationship_filter(&[])
        .expect("unfiltered traversal uses standard edges")
        .contains(&EdgeKind::RouteImport));
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

    let root = Path::new("/repo");
    let config = crate::config::v2::NoMistakesConfig::default();
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.to_path_buf(),
        paths: Vec::new(),
        paths_dir: root.to_path_buf(),
        base_url: None,
    };
    let visible = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(root, &[]);
    let filtered =
        apply_filters(entries, &args, root, &config, &tsconfig, &visible, None).unwrap();

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
        file_entrypoints_are_structured: Vec::new(),
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
        file_entrypoints_are_structured: Vec::new(),
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

include!("extra_execution.rs");
include!("extra_execution_output.rs");
