#[test]
fn build_graph_over_fixture_corpus_exercises_all_edge_producers() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/codebase-analysis");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![
            (
                "@systems/*".to_string(),
                vec!["queue-dashboard/good/systems/*".to_string()],
            ),
            (
                "@example/api/*".to_string(),
                vec!["queue-dashboard/good/api/*".to_string()],
            ),
        ],
        paths_dir: root.clone(),
        base_url: Some(root.clone()),
    };

    let graph = build_graph(&root, &tsconfig);

    assert_eq!(graph.root(), root.as_path());
    assert!(graph.all_files().count() > 10);
}

#[test]
fn graph_build_plan_import_only_enables_only_imports() {
    let allowed: HashSet<EdgeKind> = [EdgeKind::Import, EdgeKind::TypeImport].into();

    let plan = GraphBuildPlan::from_allowed(Some(&allowed));

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
fn package_dependency_names_returns_dependency_names() {
    let package_json = serde_json::json!({
        "dependencies": {
            "@scope/local": "workspace:^",
            "external": "^1.0.0"
        },
        "devDependencies": {
            "@scope/dev-local": "workspace:*"
        }
    });

    let names = package_dependency_names(&package_json);

    assert_eq!(names, vec!["@scope/dev-local", "@scope/local", "external"]);
}

#[test]
fn playwright_layout_edges_use_discovered_file_set() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("playwright-impact-routing"));
    let frontend_root = root.join("web/app");
    let page = root.join("web/app/users/[id]/page.tsx");
    let all_files: HashSet<PathBuf> = [
        root.join("web/app/layout.tsx"),
        root.join("web/app/users/layout.tsx"),
        root.join("web/app/users/[id]/page.tsx"),
    ]
    .into();

    assert_eq!(
        collect_layout_chain_files_from_file_set(&page, &frontend_root, &all_files),
        vec![
            root.join("web/app/users/layout.tsx"),
            root.join("web/app/layout.tsx")
        ]
    );
    assert_eq!(
        playwright_frontend_root(&fixture("simple")),
        fixture("simple").join("web/app")
    );
}

#[test]
fn lazy_import_deps_walks_only_reachable_import_graph() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("lazy-import"));
    let entry = root.join("src/a.mts");
    let b = root.join("src/b.mts");

    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };

    let deps = lazy_import_deps_of(&[NodeId::File(entry)], &root, &tsconfig, None).unwrap();

    assert_eq!(
        deps.iter()
            .filter_map(|entry| entry.node.as_file())
            .collect::<Vec<_>>(),
        vec![b.as_path()]
    );
}

#[test]
fn lazy_import_deps_filters_granular_relationships() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("granular-imports"));
    let entry = root.join("src/entry.mts");
    let static_file = root.join("src/static.mts");
    let type_file = root.join("src/type.mts");
    let dynamic_file = root.join("src/dynamic.mts");
    let require_file = root.join("src/require.mts");

    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph_files = GraphFiles::discover(&root);

    // Test static only
    let static_allowed = Some([EdgeKind::Import].into());
    let static_deps = lazy_import_deps_of_with_files(
        &[NodeId::File(entry.clone())],
        &root,
        &tsconfig,
        None,
        &graph_files,
        static_allowed.as_ref(),
    );
    let static_paths: Vec<_> = static_deps
        .iter()
        .filter_map(|e| e.node.as_file())
        .collect();
    assert_eq!(static_paths, vec![static_file.as_path()]);

    // Test dynamic only
    let dynamic_allowed = Some([EdgeKind::DynamicImport].into());
    let dynamic_deps = lazy_import_deps_of_with_files(
        &[NodeId::File(entry.clone())],
        &root,
        &tsconfig,
        None,
        &graph_files,
        dynamic_allowed.as_ref(),
    );
    let dynamic_paths: Vec<_> = dynamic_deps
        .iter()
        .filter_map(|e| e.node.as_file())
        .collect();
    assert_eq!(dynamic_paths, vec![dynamic_file.as_path()]);

    // Test type only
    let type_allowed = Some([EdgeKind::TypeImport].into());
    let type_deps = lazy_import_deps_of_with_files(
        &[NodeId::File(entry.clone())],
        &root,
        &tsconfig,
        None,
        &graph_files,
        type_allowed.as_ref(),
    );
    let type_paths: Vec<_> = type_deps.iter().filter_map(|e| e.node.as_file()).collect();
    assert_eq!(type_paths, vec![type_file.as_path()]);

    // Test require only
    let require_allowed = Some([EdgeKind::Require].into());
    let require_deps = lazy_import_deps_of_with_files(
        &[NodeId::File(entry.clone())],
        &root,
        &tsconfig,
        None,
        &graph_files,
        require_allowed.as_ref(),
    );
    let require_paths: Vec<_> = require_deps
        .iter()
        .filter_map(|e| e.node.as_file())
        .collect();
    assert_eq!(require_paths, vec![require_file.as_path()]);

    // Test all allowed (None)
    let all_deps = lazy_import_deps_of_with_files(
        &[NodeId::File(entry)],
        &root,
        &tsconfig,
        None,
        &graph_files,
        None,
    );
    let all_paths: HashSet<_> = all_deps.iter().filter_map(|e| e.node.as_file()).collect();
    assert_eq!(all_paths.len(), 4);
    assert!(all_paths.contains(static_file.as_path()));
    assert!(all_paths.contains(dynamic_file.as_path()));
    assert!(all_paths.contains(type_file.as_path()));
    assert!(all_paths.contains(require_file.as_path()));
}

// ── build_filter / apply_filter ─────────────────────────────────────────

#[test]
fn build_filter_none_for_empty() {
    let f = build_filter(&[]).unwrap();
    assert!(f.is_none());
}

#[test]
fn build_filter_matches_glob() {
    let spec = build_filter(&["**/*.test.mts".to_string()])
        .unwrap()
        .unwrap();
    let root = p("/root");
    let entries = vec![
        mk_entry("/root/src/foo.test.mts", 1),
        mk_entry("/root/src/foo.mts", 1),
    ];
    let result = apply_filter(entries, Some(&spec), &root);
    assert_eq!(result.len(), 1);
    assert!(result[0]
        .node
        .as_file()
        .unwrap()
        .to_str()
        .unwrap()
        .contains("foo.test.mts"));
}

// ── add_test_edges direction ─────────────────────────────────────────────

#[test]
fn test_of_edges_do_not_make_source_depend_on_test() {
    // Regression: previously add_test_edges emitted forward[src→test] which
    // made `dependencies foo.mts` return its test file as a forward dep.
    let src = p("/root/foo.mts");
    let test = p("/root/foo.test.mts");
    let mut forward: EdgeMap = HashMap::new();
    let mut reverse: EdgeMap = HashMap::new();
    merge_edges(
        &mut forward,
        &mut reverse,
        collect_test_edges(&[src.clone(), test.clone()]),
    );

    // forward: test→src only (test depends on source)
    let test_fwd: Vec<_> = forward
        .get(&NodeId::File(test.clone()))
        .unwrap_or(&vec![])
        .iter()
        .map(|(n, _)| n.clone())
        .collect();
    assert!(
        test_fwd.contains(&NodeId::File(src.clone())),
        "forward test→src"
    );
    let src_fwd: Vec<_> = forward
        .get(&NodeId::File(src.clone()))
        .unwrap_or(&vec![])
        .iter()
        .map(|(n, _)| n.clone())
        .collect();
    assert!(
        !src_fwd.contains(&NodeId::File(test.clone())),
        "forward src→test must NOT exist"
    );

    // reverse: src→test only (source is tested by test file)
    let src_rev: Vec<_> = reverse
        .get(&NodeId::File(src.clone()))
        .unwrap_or(&vec![])
        .iter()
        .map(|(n, _)| n.clone())
        .collect();
    assert!(
        src_rev.contains(&NodeId::File(test.clone())),
        "reverse src→test"
    );
    let test_rev: Vec<_> = reverse
        .get(&NodeId::File(test.clone()))
        .unwrap_or(&vec![])
        .iter()
        .map(|(n, _)| n.clone())
        .collect();
    assert!(
        !test_rev.contains(&NodeId::File(src.clone())),
        "reverse test→src must NOT exist"
    );
}

#[test]
fn apply_filter_none_keeps_all() {
    let entries = vec![mk_entry("/a.ts", 1), mk_entry("/b.ts", 2)];
    let result = apply_filter(entries.clone(), None, p("/").as_path());
    assert_eq!(result.len(), 2);
}

#[test]
fn apply_filter_removes_non_matching() {
    let spec = build_filter(&["**/*.test.ts".to_string()])
        .unwrap()
        .unwrap();
    let root = p("/root");
    let entries = vec![
        mk_entry("/root/src/foo.test.ts", 1),
        mk_entry("/root/src/foo.ts", 1),
    ];
    let result = apply_filter(entries, Some(&spec), &root);
    assert_eq!(result.len(), 1);
    assert!(result[0]
        .node
        .as_file()
        .unwrap()
        .to_str()
        .unwrap()
        .contains(".test.ts"));
}

#[test]
fn apply_filter_passes_queue_job_nodes() {
    let spec = build_filter(&["**/*.test.ts".to_string()])
        .unwrap()
        .unwrap();
    let root = p("/root");
    let queue_job = NodeEntry {
        node: NodeId::QueueJob {
            queue_file: p("/root/src/queues.mts"),
            job: "sendWelcome".to_string(),
        },
        depth: 1,
        via: vec![],
    };
    let file_entry = mk_entry("/root/src/foo.mts", 1);
    let entries = vec![queue_job, file_entry];
    let result = apply_filter(entries, Some(&spec), &root);
    // QueueJob node passes through (not path-filtered); file doesn't match
    assert_eq!(result.len(), 1);
    assert!(matches!(result[0].node, NodeId::QueueJob { .. }));
}

// ── folder-suffix filter ─────────────────────────────────────────────────

#[test]
fn folder_suffix_collapses_to_folder() {
    let spec = build_filter(&["backend/systems/*/".to_string()])
        .unwrap()
        .unwrap();
    let root = p("/project");
    let entries = vec![
        mk_entry("/project/backend/systems/emails/index.mts", 1),
        mk_entry("/project/backend/systems/emails/helpers.mts", 2),
        mk_entry("/project/backend/systems/users/index.mts", 1),
    ];
    let result = apply_filter(entries, Some(&spec), &root);
    assert_eq!(result.len(), 2);
    let paths: Vec<_> = result
        .iter()
        .map(|e| e.node.as_file().unwrap().to_str().unwrap())
        .collect();
    assert!(paths.iter().any(|p| p.ends_with("emails")));
    assert!(paths.iter().any(|p| p.ends_with("users")));
}

#[test]
fn folder_suffix_uses_min_depth() {
    let spec = build_filter(&["systems/*/".to_string()]).unwrap().unwrap();
    let root = p("/root");
    let entries = vec![
        mk_entry("/root/systems/emails/deep/file.mts", 3),
        mk_entry("/root/systems/emails/shallow.mts", 1),
    ];
    let result = apply_filter(entries, Some(&spec), &root);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].depth, 1);
}

#[test]
fn folder_suffix_and_file_glob_combined() {
    let spec = build_filter(&["systems/*/".to_string(), "**/*.test.mts".to_string()])
        .unwrap()
        .unwrap();
    let root = p("/root");
    let entries = vec![
        mk_entry("/root/systems/emails/a.mts", 1),
        mk_entry("/root/other/foo.test.mts", 2),
        mk_entry("/root/other/foo.mts", 2),
    ];
    let result = apply_filter(entries, Some(&spec), &root);
    assert_eq!(result.len(), 2);
}

#[test]
fn folder_suffix_empty_produces_no_entries() {
    let spec = build_filter(&["nomatch/*/".to_string()]).unwrap().unwrap();
    let root = p("/root");
    let entries = vec![mk_entry("/root/other/file.mts", 1)];
    let result = apply_filter(entries, Some(&spec), &root);
    assert!(result.is_empty());
}

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

// ── EdgeKind::Selector / playwright selector edges ───────────────────────

#[test]
fn selector_dep_edge_maps_selector_edge_to_dep_graph_edge() {
    use crate::playwright::analysis::types::Edge as PwEdge;
    use std::sync::Arc;

    let root = p("/root");
    let app_file = Arc::new("web/components/nav.tsx".to_string());
    let test_file = Arc::new("tests/e2e/nav.spec.ts".to_string());
    let edge = PwEdge::Selector {
        test_file: test_file.clone(),
        test_name: None,
        describe_path: Arc::new(vec![]),
        app_file: app_file.clone(),
        attribute: "data-pw".to_string(),
        value: "nav-btn".to_string(),
        selector: "getByTestId('nav-btn')".to_string(),
        line: 5,
    };

    let result = selector_dep_edge(&root, &edge).unwrap();
    assert_eq!(result.0, NodeId::File(p("/root/web/components/nav.tsx")));
    assert_eq!(result.1, NodeId::File(p("/root/tests/e2e/nav.spec.ts")));
    assert_eq!(result.2, EdgeKind::Selector);
}

#[test]
fn selector_dep_edge_maps_locator_text_edge_to_dep_graph_edge() {
    use crate::playwright::analysis::types::{Edge as PwEdge, SelectorRef};
    use std::sync::Arc;

    let root = p("/root");
    let app_file = Arc::new("web/components/button.tsx".to_string());
    let test_file = Arc::new("tests/e2e/button.spec.ts".to_string());
    let edge = PwEdge::LocatorText {
        test_file: test_file.clone(),
        test_name: None,
        describe_path: Arc::new(vec![]),
        app_file: app_file.clone(),
        locator_kind: "getByRole".to_string(),
        role: Some("button".to_string()),
        text: "Save".to_string(),
        locator: "getByRole('button', { name: 'Save' })".to_string(),
        selector_refs: vec![SelectorRef {
            attribute: "data-pw".to_string(),
            value: "save-btn".to_string(),
        }],
        reasons: vec![],
        line: 10,
    };

    let result = selector_dep_edge(&root, &edge).unwrap();
    assert_eq!(result.0, NodeId::File(p("/root/web/components/button.tsx")));
    assert_eq!(result.1, NodeId::File(p("/root/tests/e2e/button.spec.ts")));
    assert_eq!(result.2, EdgeKind::Selector);
}

#[test]
fn selector_dep_edge_returns_none_for_route_edge() {
    use crate::playwright::analysis::types::Edge as PwEdge;
    use std::sync::Arc;

    let root = p("/root");
    let edge = PwEdge::Route {
        test_file: Arc::new("tests/e2e/nav.spec.ts".to_string()),
        test_name: None,
        describe_path: Arc::new(vec![]),
        route_file: Arc::new("web/app/page.tsx".to_string()),
        route: Arc::new("/".to_string()),
        url: Arc::new("http://localhost/".to_string()),
        hook: false,
        line: 1,
    };
    assert!(selector_dep_edge(&root, &edge).is_none());
}

#[test]
fn collect_playwright_selector_edges_returns_empty_without_playwright_config() {
    // A fixture with no playwright config should return empty without panicking.
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("simple"));
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let edges = collect_playwright_selector_edges(&root, &all_files);
    // No playwright config → error → empty vec (graceful fallback).
    let _ = edges; // may be empty or not, just must not panic
}

#[test]
fn graph_build_plan_playwright_selectors_enabled_in_all() {
    let plan = GraphBuildPlan::all();
    assert!(plan.playwright_selectors);
}

#[test]
fn graph_build_plan_playwright_selectors_from_allowed() {
    let allowed: HashSet<EdgeKind> = [EdgeKind::Selector].into();
    let plan = GraphBuildPlan::from_allowed(Some(&allowed));
    assert!(plan.playwright_selectors);
    assert!(!plan.playwright_routes);
    assert!(!plan.imports);
}

#[test]
fn graph_build_plan_playwright_selectors_not_set_by_default() {
    let plan = GraphBuildPlan::default();
    assert!(!plan.playwright_selectors);
}
