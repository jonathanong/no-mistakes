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

    let names = crate::codebase::package_deps::dependency_names_from_value(
        &package_json,
        crate::codebase::package_deps::ALL_DEPENDENCY_FIELDS,
    );

    assert_eq!(names, vec!["@scope/dev-local", "@scope/local", "external"]);
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
        collect_test_edges(Path::new("/root"), &[src.clone(), test.clone()], None),
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
