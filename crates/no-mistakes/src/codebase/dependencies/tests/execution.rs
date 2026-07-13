#[test]
fn get_entries_supports_import_only_dependencies() {
    let root = fixture_root("simple");
    let raw_entrypoints = vec![PathBuf::from("a.mts")];
    let entrypoints = resolve_entrypoints(&raw_entrypoints, &root, &root);
    let roots = entrypoints
        .iter()
        .map(|ep| NodeId::File(ep.file.clone()))
        .collect::<Vec<_>>();
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph_files = graph::GraphFiles::discover(&root);
    let ctx = TraversalCtx {
        root: &root,
        tsconfig: &tsconfig,
        graph_files: &graph_files,
        build_plan: graph::GraphBuildPlan::all(),
        allowed: None,
        symbols: false,
    };

    let entries =
        get_entries(Direction::Deps, &roots, &entrypoints, None, true, &ctx).unwrap();
    assert!(!entries.is_empty());
}

#[test]
fn get_entries_supports_symbol_dependents() {
    let root = fixture_root("symbol-export");
    let entrypoints = vec![
        Entrypoint {
            file: root.join("source.mts"),
            node: NodeId::File(root.join("source.mts")),
            symbol: Some("alpha".into()),
        },
        Entrypoint {
            file: root.join("source.mts"),
            node: NodeId::File(root.join("source.mts")),
            symbol: None,
        },
    ];
    let roots = entrypoints
        .iter()
        .map(|ep| NodeId::File(ep.file.clone()))
        .collect::<Vec<_>>();
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph_files = graph::GraphFiles::discover(&root);
    let ctx = TraversalCtx {
        root: &root,
        tsconfig: &tsconfig,
        graph_files: &graph_files,
        build_plan: graph::GraphBuildPlan::all(),
        allowed: None,
        symbols: false,
    };

    let entries = get_entries(
        Direction::Dependents,
        &roots,
        &entrypoints,
        None,
        false,
        &ctx,
    )
    .unwrap();
    assert!(!entries.is_empty());
}

// ── Integration: build graph from fixture ──────────────────────────────

#[test]
fn deps_fixture_simple_json_output() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("simple")
        .join("fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);

    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let g = build_graph(&root, &tsconfig);
    let a = root.join("a.mts");
    let entries = g.deps_of(&[NodeId::File(a)], None, None);
    assert!(!entries.is_empty(), "a.mts should have deps");

    let mut buf = Vec::new();
    output::write_json(&["a.mts".to_string()], &entries, &root, &mut buf).unwrap();

    let s = String::from_utf8(buf).unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    let files = v["files"].as_array().unwrap();
    let paths: Vec<&str> = files.iter().map(|f| f["path"].as_str().unwrap()).collect();
    assert!(paths.contains(&"b.mts"), "b.mts should appear");
    assert!(paths.contains(&"c.mts"), "c.mts should appear");
}

#[test]
fn deps_fixture_format_output() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("format-output")
        .join("fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let g = build_graph(&root, &tsconfig);
    let a = root.join("a.mts");
    let entries = g.deps_of(&[NodeId::File(a)], None, None);

    // Verify md output contains backtick-quoted paths.
    let mut buf = Vec::new();
    output::write_md(&["a.mts".to_string()], &entries, &root, &mut buf).unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("`b.mts`") || s.contains("`c.mts`"));

    // Verify yml output parses correctly.
    let mut buf2 = Vec::new();
    output::write_yml(&["a.mts".to_string()], &entries, &root, &mut buf2).unwrap();
    let s2 = String::from_utf8(buf2).unwrap();
    let v: serde_yaml::Value = serde_yaml::from_str(&s2).unwrap();
    assert!(v["files"].as_sequence().unwrap().len() >= 2);
}

#[test]
fn deps_test_framework_vitest_filter() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("test-framework")
        .join("fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let g = build_graph(&root, &tsconfig);
    let idx = root.join("src").join("index.mts");
    let entries = g.dependents_of(&[NodeId::File(idx)], None, None);

    let mut filters = test_globs("vitest");
    filters.extend(test_globs("playwright"));
    let filter_spec = graph::build_filter(&filters).unwrap().unwrap();
    let filtered = graph::apply_filter(entries, Some(&filter_spec), &root);

    let paths: Vec<_> = filtered
        .iter()
        .filter_map(|e| e.node.as_file())
        .map(|p| p.to_str().unwrap())
        .collect();
    assert!(
        paths.iter().any(|p| p.ends_with("index.test.mts")),
        "vitest test should be included"
    );
}

#[test]
fn filter_fixture_excludes_test_files() {
    // fixtures/filter/src/main.mts imports both utils.mts and utils.test.mts.
    // With a glob filter of "**/*.test.mts", only test files should appear.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("filter")
        .join("fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let g = build_graph(&root, &tsconfig);
    let main = root.join("src").join("main.mts");
    let entries = g.deps_of(&[NodeId::File(main)], None, None);
    assert!(!entries.is_empty(), "main.mts should have deps");

    let filter_spec = graph::build_filter(&["**/*.test.mts".to_string()])
        .unwrap()
        .unwrap();
    let filtered = graph::apply_filter(entries, Some(&filter_spec), &root);

    let paths: Vec<_> = filtered
        .iter()
        .filter_map(|e| e.node.as_file())
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();
    assert!(
        paths.iter().all(|p| p.ends_with(".test.mts")),
        "filter should only return .test.mts files, got: {:?}",
        paths
    );
    assert!(
        paths.contains(&"utils.test.mts"),
        "expected utils.test.mts in filtered output, got: {:?}",
        paths
    );
}

#[test]
fn symbol_export_fixture_alpha_dependents() {
    // fixtures/symbol-export/source.mts exports alpha, beta, gamma.
    // uses-alpha.mts and reexport.mts both import alpha.
    // uses-beta.mts imports beta. uses-all.mts imports all three.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("symbol-export")
        .join("fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let g = build_graph(&root, &tsconfig);
    let source = root.join("source.mts");
    let entries = g.dependents_of(&[NodeId::File(source)], None, None);
    assert!(!entries.is_empty(), "source.mts should have dependents");

    let paths: Vec<_> = entries
        .iter()
        .filter_map(|e| e.node.as_file())
        .filter_map(|p| p.file_name())
        .map(|n| n.to_str().unwrap())
        .collect();
    // uses-alpha, uses-beta, uses-all, reexport, consumer all ultimately depend on source.
    assert!(
        paths.contains(&"uses-alpha.mts"),
        "expected uses-alpha.mts in dependents of source.mts, got: {:?}",
        paths
    );
    assert!(
        paths.contains(&"uses-beta.mts"),
        "expected uses-beta.mts in dependents of source.mts, got: {:?}",
        paths
    );
}

#[test]
fn folder_suffix_fixture() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join("folder-suffix")
        .join("fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let g = build_graph(&root, &tsconfig);
    let main = root.join("main.mts");
    let entries = g.deps_of(&[NodeId::File(main)], None, None);

    let spec = graph::build_filter(&["backend/systems/*/".to_string()])
        .unwrap()
        .unwrap();
    let filtered = graph::apply_filter(entries, Some(&spec), &root);

    // Should collapse to 3 folder entries: emails, users, search.
    assert_eq!(
        filtered.len(),
        3,
        "expected 3 folders, got {:?}",
        filtered
            .iter()
            .filter_map(|e| e.node.as_file())
            .map(|p| p.to_str().unwrap())
            .collect::<Vec<_>>()
    );
}
