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
    assert!(relationships_are_import_only(&[RelationshipArg::ImportStatic]));
    assert!(relationships_are_import_only(&[RelationshipArg::ImportDynamic]));
    assert!(relationships_are_import_only(&[RelationshipArg::ImportType]));
    assert!(relationships_are_import_only(&[RelationshipArg::ImportRequire]));
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
    assert_eq!(filtered[0].node, NodeId::Module("@react/client".to_string()));
}

#[test]
fn deps_direction_rejects_symbol_entrypoints() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/codebase-analysis")
        .join("simple");
    let args = TraverseArgs {
        files: vec![PathBuf::from("a.mts#a")],
        root: Some(root),
        tsconfig: None,
        depth: None,
        filters: Vec::new(),
        target_modules: Vec::new(),
        tests: Vec::new(),
        relationships: Vec::new(),
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
        .join("../../fixtures/codebase-analysis")
        .join("simple")
}

fn symbol_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/codebase-analysis")
        .join("symbol-export")
}

fn traverse_args(root: PathBuf, files: Vec<PathBuf>) -> TraverseArgs {
    TraverseArgs {
        files,
        root: Some(root),
        tsconfig: None,
        depth: Some(3),
        filters: Vec::new(),
        target_modules: Vec::new(),
        tests: Vec::new(),
        relationships: Vec::new(),
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
