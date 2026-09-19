fn prepared_bounded_shared(root: PathBuf) -> SharedTraversalContext {
    SharedTraversalContext::prepare(
        root,
        None,
        None,
        graph::GraphBuildPlan {
            imports: true,
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn apply_candidate_inventory_is_idempotent_and_skips_unbounded() {
    let root = bounded_root();
    let mut shared = prepared_bounded_shared(root.clone());
    let unbounded = import_only_args(root.clone(), vec![PathBuf::from("web/app/page.tsx")]);
    shared.apply_candidate_inventory(&unbounded).unwrap();
    assert!(!shared.candidate_inventory_applied());
    let without_bounds = shared.candidate_graph_files(&unbounded).unwrap();
    assert!(without_bounds.contains_visible(&root.join("web/lib/unrelated.test.ts")));

    let bounded = bounded_args(root, vec![PathBuf::from("web/app/page.tsx")]);
    shared.apply_candidate_inventory(&bounded).unwrap();
    assert!(shared.candidate_inventory_applied());
    shared.apply_candidate_inventory(&bounded).unwrap();
    assert!(shared.candidate_inventory_applied());
}

#[test]
fn seed_bounded_lazy_import_graph_reuses_cache_and_skips_missing_roots() {
    let root = bounded_root();
    let cwd = std::env::current_dir().unwrap();
    let mut shared = prepared_bounded_shared(root.clone());
    let args = bounded_args(root, vec![PathBuf::from("web/app/page.tsx")]);
    shared
        .seed_bounded_lazy_import_graph_from_args(&args, &cwd)
        .unwrap();
    assert!(shared.bounded_lazy_import_graph(&args).is_some());
    shared
        .seed_bounded_lazy_import_graph_from_args(&args, &cwd)
        .unwrap();
    assert!(shared.bounded_lazy_import_graph(&args).is_some());

    let mut empty_shared = prepared_bounded_shared(bounded_root());
    let empty = bounded_args(bounded_root(), Vec::new());
    empty_shared
        .seed_bounded_lazy_import_graph_from_args(&empty, &cwd)
        .unwrap();
    assert!(empty_shared.bounded_lazy_import_graph(&empty).is_none());
}

#[test]
fn scoped_seed_diagnostics_match_normalized_paths_and_skip_unresolvable() {
    let file = PathBuf::from("/repo/src/a.ts");
    let dotted = PathBuf::from("/repo/src/foo/../a.ts");
    assert_ne!(file, dotted);
    assert_eq!(crate::codebase::ts_resolver::normalize_path(&dotted), file);
    let diagnostic = crate::codebase::ts_resolver::TsConfigDiagnostic {
        kind: crate::codebase::ts_resolver::TsConfigDiagnosticKind::AmbiguousOwnership,
        config: None,
        file: Some(file),
        detail: "norm".into(),
        candidates: Vec::new(),
    };
    let missing = crate::codebase::ts_resolver::TsConfigDiagnostic {
        kind: crate::codebase::ts_resolver::TsConfigDiagnosticKind::AmbiguousOwnership,
        config: None,
        file: Some(PathBuf::from("/definitely-missing/left.ts")),
        detail: "gone".into(),
        candidates: Vec::new(),
    };
    let entries = [graph::NodeEntry {
        node: graph::NodeId::file(dotted),
        depth: 0,
        via: Vec::new(),
    }];
    let mut runtime = Vec::new();
    extend_scoped_seed_diagnostics(&mut runtime, &[diagnostic.clone(), missing], &entries);
    assert_eq!(runtime, vec![diagnostic]);
}
