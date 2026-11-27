use super::*;

fn codebase_intel() -> (PathBuf, TsConfig, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("codebase-intel"));
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let graph = build_graph(&root, &tsconfig);
    (root, tsconfig, graph)
}

fn has_file(entries: &[NodeEntry], path: &Path) -> bool {
    entries
        .iter()
        .any(|entry| entry.node.as_file() == Some(path))
}

include!("extra_integration.rs");

#[test]
fn lazy_import_handles_depth_virtual_roots_hidden_targets_and_duplicate_kinds() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("simple"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let a = root.join("a.mts");
    let b = root.join("b.mts");
    let c = root.join("c.mts");
    let hidden = root.join("hidden.mts");
    let graph_files = GraphFiles::from_parts(
        vec![a.clone(), b.clone(), c.clone(), hidden.clone()],
        vec![a.clone(), b.clone(), c.clone(), hidden],
        [a.clone(), b.clone(), c.clone()],
        vec![],
    );

    let roots = vec![NodeId::queue_job(a.clone(), "send"), NodeId::file(a)];
    let limited =
        lazy_import_deps_of_with_files(&roots, &root, &tsconfig, Some(1), &graph_files, None);
    assert!(has_file(&limited, &b));
    assert!(!has_file(&limited, &c));

    let full = lazy_import_deps_of_with_files(
        &[NodeId::file(root.join("a.mts"))],
        &root,
        &tsconfig,
        None,
        &graph_files,
        None,
    );
    assert!(has_file(&full, &b));
    assert!(has_file(&full, &c));

    let overlapping = lazy_import_deps_of_with_files(
        &[
            NodeId::file(root.join("a.mts")),
            NodeId::file(root.join("a.mts")),
            NodeId::file(b.clone()),
        ],
        &root,
        &tsconfig,
        None,
        &graph_files,
        None,
    );
    assert!(has_file(&overlapping, &c));

    let duplicate_root = crate::codebase::ts_resolver::normalize_path(&fixture("lazy-duplicates"));
    let duplicate_tsconfig = TsConfig {
        dir: duplicate_root.clone(),
        paths: vec![],
        paths_dir: duplicate_root.clone(),
        base_url: None,
    };
    let duplicate_files = GraphFiles::discover(&duplicate_root);
    let duplicate = lazy_import_deps_of_with_files(
        &[NodeId::file(duplicate_root.join("a.mts"))],
        &duplicate_root,
        &duplicate_tsconfig,
        None,
        &duplicate_files,
        None,
    );
    let duplicate_b = duplicate_root.join("b.mts");
    let b_entry = duplicate
        .iter()
        .find(|entry| entry.node.as_file() == Some(duplicate_b.as_path()))
        .unwrap();
    assert_eq!(b_entry.via, vec![EdgeKind::Import, EdgeKind::TypeImport]);

    let module_root = crate::codebase::ts_resolver::normalize_path(&fixture("graph-modules"));
    let module_tsconfig = TsConfig {
        dir: module_root.clone(),
        paths: vec![],
        paths_dir: module_root.clone(),
        base_url: None,
    };
    let module_files = GraphFiles::discover(&module_root);
    let module_deps = lazy_import_deps_of_with_files(
        &[NodeId::file(module_root.join("src/entry.mts"))],
        &module_root,
        &module_tsconfig,
        None,
        &module_files,
        Some(&[EdgeKind::Import].into()),
    );
    assert!(!module_deps
        .iter()
        .any(|entry| entry.node == NodeId::module("@local/pkg")));

    let hidden_root = crate::codebase::ts_resolver::normalize_path(&fixture("lazy-hidden"));
    let hidden_tsconfig = TsConfig {
        dir: hidden_root.clone(),
        paths: vec![],
        paths_dir: hidden_root.clone(),
        base_url: None,
    };
    let hidden_graph_files = GraphFiles::from_parts(
        vec![hidden_root.join("a.mts"), hidden_root.join("hidden.mts")],
        vec![hidden_root.join("a.mts"), hidden_root.join("hidden.mts")],
        [hidden_root.join("a.mts")],
        vec![],
    );
    assert!(lazy_import_deps_of_with_files(
        &[NodeId::file(hidden_root.join("a.mts"))],
        &hidden_root,
        &hidden_tsconfig,
        None,
        &hidden_graph_files,
        None,
    )
    .is_empty());
}

include!("extra_workspace_collectors.rs");

#[test]
fn graph_helpers_cover_test_markdown_ci_symbol_and_queue_paths() {
    let (root, tsconfig, graph) = codebase_intel();
    let emails = root.join("packages/api/src/emails.mts");
    let send_email = root.join("packages/api/src/send-email.mts");

    assert_eq!(graph.root(), root.as_path());
    assert!(graph
        .all_files()
        .any(|node| node.as_file() == Some(emails.as_path())));

    let symbol_index = SymbolIndex::build_from_root(&root, &tsconfig).unwrap();
    let dependents = graph.dependents_of_symbol(
        &emails,
        "sendWelcomeEmail",
        None,
        Some(&[EdgeKind::QueueEnqueue].into()),
        &symbol_index,
    );
    assert!(graph
        .dependents_of_symbol(
            &emails,
            "sendWelcomeEmail",
            Some(0),
            Some(&[EdgeKind::QueueEnqueue].into()),
            &symbol_index,
        )
        .is_empty());
    assert!(dependents.iter().any(|entry| {
        matches!(
            entry.node,
            NodeId::File(ref path) if path.as_ref() == send_email.as_path()
        )
    }));

    let missing_root = root.join("does-not-exist");
    let missing_tsconfig = TsConfig {
        dir: missing_root.clone(),
        paths: vec![],
        paths_dir: missing_root.clone(),
        base_url: None,
    };
    assert!(
        SymbolIndex::build_from_root(&missing_root, &missing_tsconfig)
            .unwrap()
            .importers_of(&emails, "none")
            .is_none()
    );

    let graph_files = GraphFiles::discover(&root);
    let missing = root.join("missing.md");
    assert!(collect_md_edges(
        &[missing],
        &graph_files,
        &crate::codebase::analysis_session::PathInterner::new(),
        None,
    )
    .is_empty());
    let md_edges = collect_md_edges(
        &[root.join("README.md")],
        &graph_files,
        &crate::codebase::analysis_session::PathInterner::new(),
        None,
    );
    assert!(md_edges.iter().any(|(_, to, kind)| {
        *kind == EdgeKind::MarkdownLink
            && to.as_file() == Some(root.join("packages/api/src/index.mts").as_path())
    }));

    let mut forward = EdgeMap::default();
    let mut reverse = EdgeMap::default();
    let parsed = parsed_workflow_set(&root, graph_files.all());
    add_ci_edges(
        &root,
        graph_files.all(),
        &parsed,
        &mut forward,
        &mut reverse,
        &crate::codebase::analysis_session::PathInterner::new(),
        None,
    );
    assert!(!forward.is_empty());

    let mut missing_forward = EdgeMap::default();
    let mut missing_reverse = EdgeMap::default();
    let missing_workflow = root.join(".github/workflows/missing.yml");
    let missing_files = [
        root.join("Cargo.toml"),
        root.join("src/bin/guardrails.rs"),
        missing_workflow,
        root.join(".github/workflows/not-yaml.txt"),
    ];
    let missing_parsed = parsed_workflow_set(&root, &missing_files);
    add_ci_edges(
        &root,
        &missing_files,
        &missing_parsed,
        &mut missing_forward,
        &mut missing_reverse,
        &crate::codebase::analysis_session::PathInterner::new(),
        None,
    );
    assert!(missing_forward.is_empty());

    let nested_root = crate::codebase::ts_resolver::normalize_path(&fixture("cargo-nested-bin"));
    let nested_visible: crate::fx::PathSet = [nested_root.join("src/bin/nested/main.rs")]
        .into_iter()
        .collect();
    assert_eq!(
        resolve_cargo_bin_source(&nested_root, "nested", "missing.rs", &nested_visible),
        Some(nested_root.join("src/bin/nested/main.rs"))
    );
    let mut nested_forward = EdgeMap::default();
    let mut nested_reverse = EdgeMap::default();
    let nested_files = [
        nested_root.join("Cargo.toml"),
        nested_root.join("src/bin/nested/main.rs"),
        nested_root.join(".github/workflows/bad.yml"),
    ];
    let nested_parsed = parsed_workflow_set(&nested_root, &nested_files);
    add_ci_edges(
        &nested_root,
        &nested_files,
        &nested_parsed,
        &mut nested_forward,
        &mut nested_reverse,
        &crate::codebase::analysis_session::PathInterner::new(),
        None,
    );
    assert!(nested_forward.is_empty());

    let no_workflows_root =
        crate::codebase::ts_resolver::normalize_path(&fixture("cargo-no-workflows"));
    let mut no_workflows_forward = EdgeMap::default();
    let mut no_workflows_reverse = EdgeMap::default();
    let no_workflow_files = [
        no_workflows_root.join("Cargo.toml"),
        no_workflows_root.join("src/bin/standalone.rs"),
    ];
    let no_workflow_parsed = parsed_workflow_set(&no_workflows_root, &no_workflow_files);
    add_ci_edges(
        &no_workflows_root,
        &no_workflow_files,
        &no_workflow_parsed,
        &mut no_workflows_forward,
        &mut no_workflows_reverse,
        &crate::codebase::analysis_session::PathInterner::new(),
        None,
    );
    assert!(no_workflows_forward.is_empty());
    let nested_without_workflow = [
        nested_root.join("Cargo.toml"),
        nested_root.join("src/bin/nested/main.rs"),
    ];
    let nested_without_parsed = parsed_workflow_set(&nested_root, &nested_without_workflow);
    add_ci_edges(
        &nested_root,
        &nested_without_workflow,
        &nested_without_parsed,
        &mut nested_forward,
        &mut nested_reverse,
        &crate::codebase::analysis_session::PathInterner::new(),
        None,
    );
    assert!(nested_forward.is_empty());
    let mut bins = CargoBinIndex::default();
    let empty_visible = crate::fx::PathSet::default();
    add_manifest_bins(
        Path::new("/"),
        "[[bin]]\nname = \"root\"\npath = \"main.rs\"\n",
        &empty_visible,
        &mut bins,
    );
    add_manifest_bins(
        &root.join("Cargo.toml"),
        "[[bin]",
        &empty_visible,
        &mut bins,
    );
    let outside = collect_cargo_bins(&root, &[PathBuf::from("/outside/Cargo.toml")], None);
    assert!(outside.by_name.is_empty());
    let missing_member_manifest =
        collect_cargo_bins(&root, &[root.join("missing/Cargo.toml")], None);
    assert!(missing_member_manifest.by_name.is_empty());
    let invalid_root = crate::codebase::ts_resolver::normalize_path(&fixture("cargo-invalid"));
    assert!(
        collect_cargo_bins(&invalid_root, &[invalid_root.join("Cargo.toml")], None)
            .by_name
            .is_empty()
    );
}

include!("extra_defensive.rs");
