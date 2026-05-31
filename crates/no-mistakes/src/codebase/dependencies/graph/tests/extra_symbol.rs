// ── SymbolIndex ──────────────────────────────────────────────────────────

#[test]
fn symbol_index_basic_lookup() {
    let mut map: HashMap<PathBuf, Vec<(PathBuf, String, String, bool)>> = HashMap::new();
    map.insert(
        p("/src/b.mts"),
        vec![(
            p("/src/a.mts"),
            "alpha".to_string(),
            "alpha".to_string(),
            false,
        )],
    );
    let index = SymbolIndex::build(&map);
    let importers = index
        .importers_of(p("/src/a.mts").as_path(), "alpha")
        .unwrap();
    assert_eq!(importers.len(), 1);
    assert_eq!(importers[0].0, p("/src/b.mts"));
}

#[test]
fn symbol_index_missing_returns_none() {
    let map: HashMap<PathBuf, Vec<(PathBuf, String, String, bool)>> = HashMap::new();
    let index = SymbolIndex::build(&map);
    assert!(index
        .importers_of(p("/src/a.mts").as_path(), "ghost")
        .is_none());
}

#[test]
fn symbol_index_multiple_importers() {
    let mut map: HashMap<PathBuf, Vec<(PathBuf, String, String, bool)>> = HashMap::new();
    map.insert(
        p("/b.mts"),
        vec![(p("/a.mts"), "fn1".to_string(), "fn1".to_string(), false)],
    );
    map.insert(
        p("/c.mts"),
        vec![(p("/a.mts"), "fn1".to_string(), "fn1".to_string(), false)],
    );
    let index = SymbolIndex::build(&map);
    let importers = index.importers_of(p("/a.mts").as_path(), "fn1").unwrap();
    assert_eq!(importers.len(), 2);
}

#[test]
fn graph_private_helpers_cover_noop_branches() {
    let mut visited_pairs = HashSet::new();
    let mut queue = VecDeque::new();
    let pair = (p("/src/a.mts"), "alpha".to_string());
    visited_pairs.insert(pair.clone());
    push_unvisited_symbol_pair(&mut visited_pairs, &mut queue, pair);
    assert!(queue.is_empty());

    let mut forward = EdgeMap::new();
    let mut reverse = EdgeMap::new();
    let file = p("/src/worker.mts");
    let queue_job = NodeId::QueueJob {
        queue_file: p("/src/queue.mts"),
        job: "send".to_string(),
    };
    add_distinct_worker_file_edges(&mut forward, &mut reverse, &file, &file, &queue_job);
    assert!(forward.is_empty());
    assert!(reverse.is_empty());
}

#[test]
fn symbol_node_file_helpers_render_paths() {
    let root = p("/repo");
    let symbol = NodeId::Symbol {
        file: p("/repo/src/current.mts"),
        symbol: "alpha".to_string(),
    };

    assert_eq!(symbol.as_file(), Some(p("/repo/src/current.mts").as_path()));
    assert_eq!(symbol.display_name(&root), "src/current.mts#alpha");
}

#[test]
fn symbol_edge_collection_covers_filtered_and_type_branches() {
    use crate::codebase::dependencies::extract::FunctionCall;
    use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
    use crate::codebase::ts_symbols::{Export, FileSymbols, NamedImport};

    let current = p("/repo/src/current.mts");
    let no_symbols = p("/repo/src/no-symbols.mts");
    let target = p("/repo/src/source.mts");
    let mut visible = HashSet::new();
    visible.insert(current.clone());
    visible.insert(no_symbols.clone());
    visible.insert(target.clone());
    let tsconfig = TsConfig {
        dir: p("/repo"),
        paths: vec![],
        paths_dir: p("/repo"),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig).with_visible(&visible);

    let symbols = FileSymbols {
        exports: vec![
            Export {
                name: "*".to_string(),
                kind: ExportKind::ReExport {
                    source: "./source.mts".to_string(),
                    imported: "*".to_string(),
                },
                line: 1,
                is_type_only: false,
            },
            Export {
                name: "Star".to_string(),
                kind: ExportKind::ReExport {
                    source: "./source.mts".to_string(),
                    imported: "*".to_string(),
                },
                line: 2,
                is_type_only: false,
            },
            Export {
                name: "Alias".to_string(),
                kind: ExportKind::ReExport {
                    source: "./source.mts".to_string(),
                    imported: "SourceType".to_string(),
                },
                line: 3,
                is_type_only: true,
            },
            Export {
                name: "run".to_string(),
                kind: ExportKind::Function,
                line: 4,
                is_type_only: false,
            },
        ],
        imports: vec![
            NamedImport {
                source: "./source.mts".to_string(),
                imported: "*".to_string(),
                local: "all".to_string(),
                line: 4,
                is_type_only: false,
            },
            NamedImport {
                source: "./source.mts".to_string(),
                imported: "used".to_string(),
                local: "used".to_string(),
                line: 5,
                is_type_only: false,
            },
        ],
    };
    let mut facts = TsFactMap::new();
    facts.insert(no_symbols.clone(), TsFileFacts::default());
    facts.insert(
        current.clone(),
        TsFileFacts {
            symbols: Some(symbols),
            function_calls: vec![
                FunctionCall {
                    caller: None,
                    callee: "used".to_string(),
                },
                FunctionCall {
                    caller: Some("helper".to_string()),
                    callee: "used".to_string(),
                },
                FunctionCall {
                    caller: Some("run".to_string()),
                    callee: "missing".to_string(),
                },
                FunctionCall {
                    caller: Some("run".to_string()),
                    callee: "used".to_string(),
                },
            ],
            ..TsFileFacts::default()
        },
    );

    let edges = collect_symbol_edges(
        &[p("/repo/src/missing.mts"), no_symbols, current.clone()],
        &facts,
        &resolver,
    );

    assert!(edges.contains(&(
        NodeId::File(current.clone()),
        NodeId::Symbol {
            file: current.clone(),
            symbol: "Alias".to_string(),
        },
        EdgeKind::TypeImport
    )));
    assert!(edges.contains(&(
        NodeId::Symbol {
            file: current.clone(),
            symbol: "Alias".to_string(),
        },
        NodeId::Symbol {
            file: target.clone(),
            symbol: "SourceType".to_string(),
        },
        EdgeKind::TypeImport
    )));
    assert!(edges.contains(&(
        NodeId::Symbol {
            file: current,
            symbol: "run".to_string(),
        },
        NodeId::Symbol {
            file: target,
            symbol: "used".to_string(),
        },
        EdgeKind::Import
    )));
}

#[test]
fn symbol_bfs_skips_initial_owner_and_honors_limits() {
    let owner = p("/repo/src/owner.mts");
    let dep = p("/repo/src/dep.mts");
    let symbol = NodeId::Symbol {
        file: owner.clone(),
        symbol: "alpha".to_string(),
    };
    let mut edges = EdgeMap::new();
    edges.insert(
        symbol.clone(),
        vec![
            (NodeId::File(owner.clone()), EdgeKind::Import),
            (NodeId::File(dep.clone()), EdgeKind::Require),
        ],
    );

    let import_only: HashSet<_> = [EdgeKind::Import].into();
    let filtered = bfs_skipping_initial_symbol_owner_files(
        std::slice::from_ref(&symbol),
        &edges,
        None,
        Some(&import_only),
    );
    assert!(filtered.is_empty());

    let unfiltered =
        bfs_skipping_initial_symbol_owner_files(std::slice::from_ref(&symbol), &edges, None, None);
    assert_eq!(unfiltered.len(), 1);
    assert_eq!(unfiltered[0].node, NodeId::File(dep));

    let limited = bfs_skipping_initial_symbol_owner_files(
        std::slice::from_ref(&symbol),
        &edges,
        Some(0),
        None,
    );
    assert!(limited.is_empty());

    let file_start = NodeId::File(owner);
    let empty = bfs_skipping_initial_symbol_owner_files(&[file_start], &edges, Some(0), None);
    assert!(empty.is_empty());
}

// ── add_test_edges ───────────────────────────────────────────────────────

#[test]
fn test_edges_source_finds_test_file() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("test-framework")
        .join("fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = build_graph(&root, &tsconfig);

    let index_mts = root.join("src/index.mts");
    let index_test = root.join("src/index.test.mts");
    let testof_filter: HashSet<EdgeKind> = [EdgeKind::TestOf].into();

    // dependents_of (reverse walk): test file is a dependent of its source.
    let dependents = graph.dependents_of(
        &[NodeId::File(index_mts.clone())],
        None,
        Some(&testof_filter),
    );
    assert!(
        dependents
            .iter()
            .any(|e| e.node.as_file() == Some(index_test.as_path())),
        "index.test.mts should appear as a dependent of index.mts"
    );

    // deps_of (forward walk): source file must NOT forward-depend on its test.
    let deps = graph.deps_of(&[NodeId::File(index_mts)], None, Some(&testof_filter));
    assert!(
        !deps
            .iter()
            .any(|e| e.node.as_file() == Some(index_test.as_path())),
        "index.mts must NOT forward-depend on index.test.mts"
    );
}

// ── add_md_edges ─────────────────────────────────────────────────────────

#[test]
fn md_edges_added_for_codebase_intel_fixture() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("codebase-intel")
        .join("fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = build_graph(&root, &tsconfig);

    let readme = root.join("README.md");
    let deps = graph.deps_of(
        &[NodeId::File(readme)],
        None,
        Some(&[EdgeKind::MarkdownLink].into()),
    );
    // README.md links to packages/api/src/index.mts
    let linked_file = root
        .join("packages")
        .join("api")
        .join("src")
        .join("index.mts");
    assert!(
        deps.iter()
            .any(|e| e.node.as_file() == Some(linked_file.as_path())),
        "README.md should have MarkdownLink edge to packages/api/src/index.mts"
    );
}
