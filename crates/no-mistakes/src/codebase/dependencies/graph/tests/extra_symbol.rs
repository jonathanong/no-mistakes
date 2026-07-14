// ── SymbolIndex ──────────────────────────────────────────────────────────

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
    test_support::add_distinct_worker_file_edges(
        &mut forward,
        &mut reverse,
        &file,
        &file,
        &queue_job,
    );
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
                local: None,
                kind: ExportKind::ReExport {
                    source: "./source.mts".to_string(),
                    imported: "*".to_string(),
                },
                line: 1,
                is_type_only: false,
            },
            Export {
                name: "Star".to_string(),
                local: None,
                kind: ExportKind::ReExport {
                    source: "./source.mts".to_string(),
                    imported: "*".to_string(),
                },
                line: 2,
                is_type_only: false,
            },
            Export {
                name: "Alias".to_string(),
                local: None,
                kind: ExportKind::ReExport {
                    source: "./source.mts".to_string(),
                    imported: "SourceType".to_string(),
                },
                line: 3,
                is_type_only: true,
            },
            Export {
                name: "run".to_string(),
                local: None,
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
                    static_arg: None,
                    static_cwd: None,
                },
                FunctionCall {
                    caller: Some("helper".to_string()),
                    callee: "used".to_string(),
                    static_arg: None,
                    static_cwd: None,
                },
                FunctionCall {
                    caller: Some("run".to_string()),
                    callee: "missing".to_string(),
                    static_arg: None,
                    static_cwd: None,
                },
                FunctionCall {
                    caller: Some("run".to_string()),
                    callee: "used".to_string(),
                    static_arg: None,
                    static_cwd: None,
                },
            ],
            symbol_references: vec![FunctionCall {
                caller: Some("run".to_string()),
                callee: "used".to_string(),
                static_arg: None,
                static_cwd: None,
            }],
            ..TsFileFacts::default()
        },
    );

    let edges = collect_symbol_edges(
        Path::new("/repo"),
        SymbolGraphFiles {
            indexable: &[p("/repo/src/missing.mts"), no_symbols, current.clone()],
            all: std::slice::from_ref(&current),
            visible: &visible,
        },
        &facts,
        &resolver,
        &Default::default(),
        None,
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
fn symbol_import_target_helpers_cover_node_kinds() {
    use crate::codebase::dependencies::extract::{ExtractedImport, ImportKind};

    let current = p("/repo/src/current.mts");
    let source = p("/repo/src/source.mts");
    let asset = p("/repo/src/data.json");
    let mut visible = HashSet::new();
    visible.insert(current.clone());
    visible.insert(source.clone());
    visible.insert(asset.clone());
    let tsconfig = TsConfig {
        dir: p("/repo"),
        paths: vec![],
        paths_dir: p("/repo"),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig).with_visible(&visible);
    let workspace = crate::codebase::workspaces::WorkspaceMap::default();

    assert_eq!(
        import_target(
            "./source.mts",
            ImportKind::Static,
            &current,
            &resolver,
            &workspace,
            &visible,
        ),
        Some((NodeId::File(source.clone()), EdgeKind::Import))
    );
    assert_eq!(
        import_target(
            "./source.mts",
            ImportKind::Type,
            &current,
            &resolver,
            &workspace,
            &visible,
        ),
        Some((NodeId::File(source.clone()), EdgeKind::TypeImport))
    );
    assert_eq!(
        import_target(
            "./source.mts",
            ImportKind::Require,
            &current,
            &resolver,
            &workspace,
            &visible,
        ),
        Some((NodeId::File(source), EdgeKind::Require))
    );
    assert_eq!(
        import_target(
            "./data.json",
            ImportKind::Static,
            &current,
            &resolver,
            &workspace,
            &visible,
        ),
        Some((NodeId::File(asset), EdgeKind::AssetImport))
    );
    assert_eq!(
        import_target(
            "zod",
            ImportKind::Dynamic,
            &current,
            &resolver,
            &workspace,
            &visible,
        ),
        Some((NodeId::Module("zod".to_string()), EdgeKind::DynamicImport))
    );
    assert_eq!(
        import_target(
            "./missing.mts",
            ImportKind::Static,
            &current,
            &resolver,
            &workspace,
            &visible,
        ),
        None
    );

    let scoped = scoped_import_map(
        &[
            ExtractedImport {
                specifier: "./source.mts".to_string(),
                kind: ImportKind::Static,
                line: 1,
                function_scope: Some("run".to_string()),
                side_effect_only: false,
                re_export: false,
                runtime_reachable: false,
            },
            ExtractedImport {
                specifier: "./missing.mts".to_string(),
                kind: ImportKind::Static,
                line: 1,
                function_scope: Some("run".to_string()),
                side_effect_only: false,
                re_export: false,
                runtime_reachable: false,
            },
            ExtractedImport {
                specifier: "react".to_string(),
                kind: ImportKind::Type,
                line: 1,
                function_scope: None,
                side_effect_only: false,
                re_export: false,
                runtime_reachable: false,
            },
        ],
        &current,
        &resolver,
        &workspace,
        &visible,
    );

    assert_eq!(
        scoped.get("run"),
        Some(&vec![(
            NodeId::File(p("/repo/src/source.mts")),
            EdgeKind::Import
        )])
    );
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
    let filtered = bfs_skipping_symbol_owner_files(
        std::slice::from_ref(&symbol),
        &edges,
        None,
        Some(&import_only),
    );
    assert!(filtered.is_empty());

    let unfiltered =
        bfs_skipping_symbol_owner_files(std::slice::from_ref(&symbol), &edges, None, None);
    assert_eq!(unfiltered.len(), 1);
    assert_eq!(unfiltered[0].node, NodeId::File(dep));

    let limited =
        bfs_skipping_symbol_owner_files(std::slice::from_ref(&symbol), &edges, Some(0), None);
    assert!(limited.is_empty());

    let file_start = NodeId::File(owner);
    let empty = bfs_skipping_symbol_owner_files(&[file_start], &edges, Some(0), None);
    assert!(empty.is_empty());
}

#[test]
fn symbol_bfs_skips_only_the_current_symbol_owner_file() {
    let owner_a = p("/repo/src/a.mts");
    let owner_b = p("/repo/src/b.mts");
    let symbol_a = NodeId::Symbol {
        file: owner_a.clone(),
        symbol: "alpha".to_string(),
    };
    let symbol_b = NodeId::Symbol {
        file: owner_b.clone(),
        symbol: "beta".to_string(),
    };
    let mut edges = EdgeMap::new();
    edges.insert(
        symbol_a.clone(),
        vec![
            (NodeId::File(owner_a.clone()), EdgeKind::Import),
            (NodeId::File(owner_b.clone()), EdgeKind::Import),
        ],
    );
    edges.insert(
        symbol_b.clone(),
        vec![(NodeId::File(owner_b.clone()), EdgeKind::Import)],
    );

    let result = bfs_skipping_symbol_owner_files(&[symbol_a, symbol_b], &edges, None, None);
    let nodes: Vec<_> = result.into_iter().map(|entry| entry.node).collect();
    assert!(!nodes.contains(&NodeId::File(owner_a)));
    assert!(nodes.contains(&NodeId::File(owner_b)));
}

#[test]
fn symbol_bfs_widens_reached_symbols_to_owner_files() {
    let source = p("/repo/src/source.mts");
    let owner = p("/repo/src/owner.mts");
    let unrelated_consumer = p("/repo/src/unrelated-consumer.mts");
    let source_symbol = NodeId::Symbol {
        file: source,
        symbol: "alpha".to_string(),
    };
    let owner_symbol = NodeId::Symbol {
        file: owner.clone(),
        symbol: "usesAlpha".to_string(),
    };
    let mut edges = EdgeMap::new();
    edges.insert(
        source_symbol.clone(),
        vec![(owner_symbol.clone(), EdgeKind::Import)],
    );
    edges.insert(
        owner_symbol.clone(),
        vec![(NodeId::File(owner.clone()), EdgeKind::Import)],
    );
    edges.insert(
        NodeId::File(owner.clone()),
        vec![(NodeId::File(unrelated_consumer.clone()), EdgeKind::Import)],
    );

    let result = bfs_skipping_symbol_owner_files(&[source_symbol], &edges, None, None);
    let nodes: Vec<_> = result.into_iter().map(|entry| entry.node).collect();

    assert!(nodes.contains(&owner_symbol));
    assert!(nodes.contains(&NodeId::File(owner)));
    assert!(nodes.contains(&NodeId::File(unrelated_consumer)));
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
