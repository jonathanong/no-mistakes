#[test]
fn cargo_ci_edges_exclude_ignored_manifests_and_bin_targets() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass3-visibility");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let graph_files = GraphFiles::discover(&root);

    let bins = collect_cargo_bins(&root, graph_files.all());
    assert_eq!(
        bins.by_name.get("visible"),
        Some(&root.join("src/bin/visible.rs"))
    );
    assert!(!bins.by_name.contains_key("ignored"));
    assert!(!bins.by_name.contains_key("member"));

    let ignored_root = root.join("ignored-root");
    let ignored_root_files = GraphFiles::discover(&ignored_root);
    assert!(collect_cargo_bins(&ignored_root, ignored_root_files.all())
        .by_name
        .is_empty());

    let mut forward = EdgeMap::new();
    let mut reverse = EdgeMap::new();
    add_ci_edges(&root, graph_files.all(), &mut forward, &mut reverse);
    let workflow = NodeId::File(root.join(".github/workflows/ci.yml"));
    let targets = forward.get(&workflow).cloned().unwrap_or_default();
    assert!(targets.iter().any(|(target, kind)| {
        *kind == EdgeKind::CiInvocation
            && target.as_file() == Some(root.join("src/bin/visible.rs").as_path())
    }));
    assert!(targets.iter().all(|(target, _)| {
        target.as_file() != Some(root.join("src/bin/ignored.rs").as_path())
            && target.as_file() != Some(root.join("crates/ignored/src/main.rs").as_path())
    }));
}

#[test]
fn process_spawn_edges_exclude_ignored_targets_from_file_and_symbol_graphs() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass3-visibility");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let allowed = HashSet::from([EdgeKind::ProcessSpawn]);
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan::from_allowed(Some(&allowed)).with_symbols(true),
    )
    .unwrap();
    let ignored = root.join("ignored-worker.ts");

    assert!(graph.all_files().all(|node| node.as_file() != Some(&ignored)));
    assert!(graph
        .dependencies_of_node(&NodeId::File(root.join("spawn.ts")))
        .into_iter()
        .flatten()
        .all(|(target, _)| target.as_file() != Some(&ignored)));
}

#[test]
fn public_lazy_import_traversal_honors_only_the_explicit_ignored_root() {
    let fixture = crate::test_support::materialize_gitignore_fixture("prepared-tsconfig");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: Vec::new(),
        paths_dir: root.clone(),
        base_url: None,
    };
    let entries = lazy_import_deps_of(
        &[NodeId::File(root.join("ignored-explicit/effect-entry.ts"))],
        &root,
        &tsconfig,
        None,
    )
    .unwrap();
    let paths = entries
        .iter()
        .filter_map(|entry| entry.node.as_file())
        .collect::<Vec<_>>();

    assert!(paths.contains(&root.join("src/effect.ts").as_path()));
    assert!(!paths.contains(&root.join("ignored-transitive/effect.ts").as_path()));
}
