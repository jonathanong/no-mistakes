use super::*;

mod config_edges;
mod config_plan;
mod extra;

fn p(s: &str) -> PathBuf {
    PathBuf::from(s)
}

fn n(s: &str) -> NodeId {
    NodeId::File(p(s))
}

fn raw_fwd(pairs: &[(&str, &[&str])]) -> HashMap<PathBuf, Vec<PathBuf>> {
    pairs
        .iter()
        .map(|(k, vs)| (p(k), vs.iter().map(|v| p(v)).collect()))
        .collect()
}

fn raw_rev(pairs: &[(&str, &[&str])]) -> HashMap<PathBuf, Vec<PathBuf>> {
    raw_fwd(pairs)
}

fn mk_entry(path: &str, depth: usize) -> NodeEntry {
    NodeEntry {
        node: NodeId::File(p(path)),
        depth,
        via: vec![],
    }
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/codebase-analysis")
        .join(name)
}

fn build_graph(root: &Path, tsconfig: &TsConfig) -> DepGraph {
    DepGraph::build_with_plan(root, tsconfig, GraphBuildPlan::all()).unwrap()
}

#[test]
fn node_display_and_normalization_cover_file_and_queue_nodes() {
    let root = p("/repo");
    let file = NodeId::File(p("/repo/src/file.ts"));
    let queue = NodeId::QueueJob {
        queue_file: p("/repo/src/queues.ts"),
        job: "send".to_string(),
    };

    assert_eq!(file.display_name(&root), "src/file.ts");
    assert_eq!(queue.display_name(&root), "src/queues.ts#send");
    assert!(file.as_file().is_some());
    assert!(queue.as_file().is_none());

    let nodes = normalize_nodes(&[file, queue]);
    assert_eq!(nodes.len(), 2);
    assert!(matches!(nodes[1], NodeId::QueueJob { .. }));
}

#[test]
fn graph_build_plan_from_allowed_covers_each_edge_family() {
    assert!(GraphBuildPlan::all().imports);
    assert!(GraphBuildPlan::all().workspace);

    let allowed: HashSet<_> = [
        EdgeKind::TypeImport,
        EdgeKind::WorkspaceImport,
        EdgeKind::TestOf,
        EdgeKind::MarkdownLink,
        EdgeKind::CiInvocation,
        EdgeKind::RouteRef,
        EdgeKind::QueueEnqueue,
        EdgeKind::QueueWorker,
        EdgeKind::RouteTest,
        EdgeKind::HttpCall,
        EdgeKind::ProcessSpawn,
    ]
    .into();
    let plan = GraphBuildPlan::from_allowed(Some(&allowed));
    assert!(plan.imports);
    assert!(plan.workspace);
    assert!(plan.tests);
    assert!(plan.markdown);
    assert!(plan.ci);
    assert!(plan.routes);
    assert!(plan.queues);
    assert!(plan.playwright_routes);
    assert!(plan.http);
    assert!(plan.process);

    let import_only: HashSet<_> = [EdgeKind::Require].into();
    let plan = GraphBuildPlan::from_allowed(Some(&import_only));
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
}

// ── bfs ─────────────────────────────────────────────────────────────────

#[test]
fn bfs_linear_chain() {
    let mut fwd: EdgeMap = HashMap::new();
    fwd.insert(n("/a"), vec![(n("/b"), EdgeKind::Import)]);
    fwd.insert(n("/b"), vec![(n("/c"), EdgeKind::Import)]);
    fwd.insert(n("/c"), vec![]);

    let entries = bfs(&[n("/a")], &fwd, None, None);
    let paths: Vec<_> = entries.iter().map(|e| e.node.as_file().unwrap()).collect();
    assert_eq!(paths, [p("/b").as_path(), p("/c").as_path()]);
    assert_eq!(entries[0].depth, 1);
    assert_eq!(entries[1].depth, 2);
    assert_eq!(entries[0].via, vec![EdgeKind::Import]);
}

#[test]
fn bfs_depth_limit() {
    let mut fwd: EdgeMap = HashMap::new();
    fwd.insert(n("/a"), vec![(n("/b"), EdgeKind::Import)]);
    fwd.insert(n("/b"), vec![(n("/c"), EdgeKind::Import)]);
    fwd.insert(n("/c"), vec![]);

    let entries = bfs(&[n("/a")], &fwd, Some(1), None);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].node.as_file().unwrap(), p("/b").as_path());
}

#[test]
fn bfs_diamond_no_duplicates() {
    let mut fwd: EdgeMap = HashMap::new();
    fwd.insert(
        n("/a"),
        vec![(n("/b"), EdgeKind::Import), (n("/c"), EdgeKind::Import)],
    );
    fwd.insert(n("/b"), vec![(n("/d"), EdgeKind::Import)]);
    fwd.insert(n("/c"), vec![(n("/d"), EdgeKind::Import)]);
    fwd.insert(n("/d"), vec![]);

    let entries = bfs(&[n("/a")], &fwd, None, None);
    let paths: Vec<_> = entries.iter().map(|e| e.node.as_file().unwrap()).collect();
    let unique: HashSet<_> = paths.iter().collect();
    assert_eq!(paths.len(), unique.len(), "no duplicates");
    assert!(entries.iter().any(|e| e.node == n("/d")));
}

#[test]
fn bfs_multiple_roots() {
    let mut fwd: EdgeMap = HashMap::new();
    fwd.insert(n("/a"), vec![(n("/c"), EdgeKind::Import)]);
    fwd.insert(n("/b"), vec![(n("/d"), EdgeKind::Import)]);
    fwd.insert(n("/c"), vec![]);
    fwd.insert(n("/d"), vec![]);

    let entries = bfs(&[n("/a"), n("/b")], &fwd, None, None);
    assert_eq!(entries.len(), 2);
}

#[test]
fn bfs_cycle_terminates() {
    let mut fwd: EdgeMap = HashMap::new();
    fwd.insert(n("/a"), vec![(n("/b"), EdgeKind::Import)]);
    fwd.insert(n("/b"), vec![(n("/a"), EdgeKind::Import)]);

    let entries = bfs(&[n("/a")], &fwd, None, None);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].node.as_file().unwrap(), p("/b").as_path());
}

#[test]
fn bfs_empty_starts() {
    let fwd: EdgeMap = HashMap::new();
    let entries = bfs(&[], &fwd, None, None);
    assert!(entries.is_empty());
}

#[test]
fn bfs_node_with_no_edges() {
    let mut fwd: EdgeMap = HashMap::new();
    fwd.insert(n("/a"), vec![]);
    let entries = bfs(&[n("/a")], &fwd, None, None);
    assert!(entries.is_empty());
}

#[test]
fn bfs_relationship_filter_excludes_wrong_kind() {
    let mut fwd: EdgeMap = HashMap::new();
    fwd.insert(
        n("/a"),
        vec![(n("/b"), EdgeKind::Import), (n("/c"), EdgeKind::TestOf)],
    );
    fwd.insert(n("/b"), vec![]);
    fwd.insert(n("/c"), vec![]);

    let allowed: HashSet<EdgeKind> = [EdgeKind::Import].into();
    let entries = bfs(&[n("/a")], &fwd, None, Some(&allowed));
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].node.as_file().unwrap(), p("/b").as_path());
}

#[test]
fn bfs_via_accumulated_from_two_paths() {
    // a → b via Import; a → b via TestOf (same destination, different kinds)
    let mut fwd: EdgeMap = HashMap::new();
    fwd.insert(
        n("/a"),
        vec![(n("/b"), EdgeKind::Import), (n("/b"), EdgeKind::TestOf)],
    );
    fwd.insert(n("/b"), vec![]);

    let entries = bfs(&[n("/a")], &fwd, None, None);
    assert_eq!(entries.len(), 1);
    // via should contain both kinds
    assert!(entries[0].via.contains(&EdgeKind::Import));
    assert!(entries[0].via.contains(&EdgeKind::TestOf));
}

// ── DepGraph::from_raw_maps ──────────────────────────────────────────────

#[test]
fn dep_graph_deps_of() {
    let fwd = raw_fwd(&[("/root/a.mts", &["/root/b.mts"]), ("/root/b.mts", &[])]);
    let rev = raw_rev(&[]);
    let g = test_support::from_raw_maps(p("/root"), fwd, rev);
    let entries = g.deps_of(&[NodeId::File(p("/root/a.mts"))], None, None);
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].node.as_file().unwrap(),
        p("/root/b.mts").as_path()
    );
}

#[test]
fn dep_graph_dependents_of() {
    let fwd = raw_fwd(&[]);
    let rev = raw_rev(&[("/root/b.mts", &["/root/a.mts"])]);
    let g = test_support::from_raw_maps(p("/root"), fwd, rev);
    let entries = g.dependents_of(&[NodeId::File(p("/root/b.mts"))], None, None);
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].node.as_file().unwrap(),
        p("/root/a.mts").as_path()
    );
}

// ── DepGraph::build integration ─────────────────────────────────────────

#[test]
fn build_graph_from_fixture() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/codebase-analysis")
        .join("simple");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build(&root, &tsconfig).unwrap();

    let a = root.join("a.mts");
    let b = root.join("b.mts");
    let c = root.join("c.mts");

    let deps_a = graph.deps_of(&[NodeId::File(a.clone())], None, None);
    let dep_paths: Vec<_> = deps_a.iter().filter_map(|e| e.node.as_file()).collect();
    assert!(dep_paths.contains(&b.as_path()));
    assert!(dep_paths.contains(&c.as_path()));

    let dependents_c = graph.dependents_of(&[NodeId::File(c.clone())], None, None);
    let dep_paths: Vec<_> = dependents_c
        .iter()
        .filter_map(|e| e.node.as_file())
        .collect();
    assert!(dep_paths.contains(&b.as_path()));
    assert!(dep_paths.contains(&a.as_path()));
}

#[test]
fn build_graph_aliased_fixture() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/codebase-analysis")
        .join("aliased");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let tsconfig_path = root.join("tsconfig.json");
    let tsconfig = crate::codebase::ts_resolver::load_tsconfig(&tsconfig_path).unwrap();
    let graph = build_graph(&root, &tsconfig);

    let main = root.join("main.mts");
    let helpers = root.join("utils").join("helpers.mts");

    let deps = graph.deps_of(&[NodeId::File(main)], None, None);
    let dep_paths: Vec<_> = deps.iter().filter_map(|e| e.node.as_file()).collect();
    assert!(dep_paths.contains(&helpers.as_path()));
}

#[test]
fn ci_edges_include_workspace_member_bins() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/codebase-analysis")
        .join("cargo-workspace-ci");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = build_graph(&root, &tsconfig);

    let workflow = root.join(".github").join("workflows").join("ci.yml");
    let implicit_main = root
        .join("crates")
        .join("tool-one")
        .join("src")
        .join("main.rs");
    let hyphenated_bin = root
        .join("crates")
        .join("pg-schema")
        .join("src")
        .join("bin")
        .join("pg-schema.rs");
    let package_scoped_bin = root
        .join("crates")
        .join("tool-one")
        .join("src")
        .join("bin")
        .join("side-tool.rs");
    let colliding_bin = root
        .join("crates")
        .join("pg-schema")
        .join("src")
        .join("bin")
        .join("side-tool.rs");
    let excluded_main = root
        .join("crates")
        .join("excluded")
        .join("src")
        .join("main.rs");
    let deps = graph.deps_of(
        &[NodeId::File(workflow)],
        None,
        Some(&[EdgeKind::CiInvocation].into()),
    );
    assert!(
        deps.iter()
            .any(|e| e.node.as_file() == Some(implicit_main.as_path())),
        "cargo run -p should link to the member's implicit src/main.rs"
    );
    assert!(
        deps.iter()
            .any(|e| e.node.as_file() == Some(hyphenated_bin.as_path())),
        "cargo run --bin should link to a hyphenated default bin path"
    );
    assert!(
        deps.iter()
            .any(|e| e.node.as_file() == Some(package_scoped_bin.as_path())),
        "cargo run -p <pkg> --bin <bin> should link to that package's bin"
    );
    assert!(
        !deps
            .iter()
            .any(|e| e.node.as_file() == Some(colliding_bin.as_path())),
        "package-qualified --bin should not link to another package's same-named bin"
    );
    assert!(
        !deps
            .iter()
            .any(|e| e.node.as_file() == Some(excluded_main.as_path())),
        "workspace exclude entries should not contribute CI bin edges"
    );
}

#[test]
fn ci_edges_include_implicit_workspace_member_bins() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("cargo-implicit-members"));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = build_graph(&root, &tsconfig);

    let workflow = root.join(".github").join("workflows").join("ci.yml");
    let implicit_main = root
        .join("crates")
        .join("implicit-tool")
        .join("src")
        .join("main.rs");
    let deps = graph.deps_of(
        &[NodeId::File(workflow)],
        None,
        Some(&[EdgeKind::CiInvocation].into()),
    );
    assert!(
        deps.iter()
            .any(|e| e.node.as_file() == Some(implicit_main.as_path())),
        "workspace member bins should be discovered even when members is omitted"
    );
}

#[test]
fn invalid_cargo_workspace_member_globs_are_ignored() {
    assert!(cargo_member_globset(&["[".to_string()]).is_none());
}

#[test]
fn build_graph_excludes_skipped_fixture_files() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("skipped-files"));
    let source = root.join("src/source.mts");
    let visible = root.join("src/visible.mts");
    let skipped = root.join("fixtures/hidden.mts");

    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = build_graph(&root, &tsconfig);

    let dependents = graph.dependents_of(&[NodeId::File(source)], None, None);
    let paths: Vec<_> = dependents.iter().filter_map(|e| e.node.as_file()).collect();
    assert_eq!(paths, vec![visible.as_path()]);
    assert!(!paths.contains(&skipped.as_path()));
}

#[test]
fn test_graph_methods_lazy() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("skipped-files"));
    let source = root.join("src/source.mts");
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = build_graph(&root, &tsconfig);

    let node = NodeId::File(source);
    let deps = graph.dependencies_of_node(&node);
    let deps_none = graph.dependencies_of_node(&NodeId::File(PathBuf::from("/nonexistent")));
    let deps_none_2 = graph.dependents_of_node(&NodeId::File(PathBuf::from("/nonexistent")));

    assert!(deps_none.is_none());
    assert!(deps_none_2.is_none());
    if let Some(deps) = deps {
        for (_dep, kind) in deps {
            assert!(matches!(
                kind,
                EdgeKind::Import | EdgeKind::TypeImport | EdgeKind::DynamicImport | EdgeKind::Require
            ));
        }
    }
}
