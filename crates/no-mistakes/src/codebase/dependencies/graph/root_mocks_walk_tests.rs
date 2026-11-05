use super::*;

fn graph() -> DepGraph {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/runtime-import-sharing"),
    );
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            imports: true,
            calls: true,
            mock_boundaries: true,
            ..Default::default()
        },
    )
    .unwrap()
}

fn walk(graph: &DepGraph, roots: &[&str], depth: Option<usize>) -> (Vec<String>, u64) {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    let roots: Vec<_> = roots
        .iter()
        .map(|file| NodeId::file(graph.root.join(file)))
        .collect();
    let files = graph.runtime_import_sources_with_root_mocks(&roots, depth);
    (
        files
            .iter()
            .map(|file| file.file_name().unwrap().to_str().unwrap().to_owned())
            .collect(),
        observer
            .snapshot()
            .work
            .get("graph.runtime_import_visits")
            .copied()
            .unwrap_or(0),
    )
}

#[test]
fn shared_import_subtree_is_visited_once_per_boundary_group() {
    let graph = graph();
    let (files, visits) = walk(&graph, &["a.ts", "b.ts"], None);
    assert_eq!(files, ["a.ts", "b.ts", "leaf.ts", "shared.ts"]);
    assert_eq!(visits, 4);
    let (files, visits) = walk(&graph, &["mocked.ts", "mocked-again.ts"], None);
    assert_eq!(files, ["mocked-again.ts", "mocked.ts", "shared.ts"]);
    assert_eq!(visits, 3);
    let (files, visits) = walk(&graph, &["a.ts", "mocked.ts"], None);
    assert_eq!(files, ["a.ts", "leaf.ts", "mocked.ts", "shared.ts"]);
    assert_eq!(visits, 5);
}

#[test]
fn shared_import_walk_preserves_depth() {
    let graph = graph();
    for depth in [0, 1] {
        assert_eq!(
            walk(&graph, &["a.ts", "b.ts"], Some(depth)),
            (vec!["a.ts".into(), "b.ts".into()], 2)
        );
    }
    assert_eq!(
        walk(&graph, &["a.ts", "shared.ts"], Some(2)),
        (vec!["a.ts".into(), "leaf.ts".into(), "shared.ts".into()], 3)
    );
}

#[test]
fn runtime_import_walk_excludes_type_edges_and_reverse_importers() {
    let graph = graph();
    assert_eq!(
        walk(&graph, &["type-only.ts"], None),
        (vec!["type-only.ts".into()], 1)
    );
    let (files, visits) = walk(&graph, &["a.ts"], None);
    assert_eq!(files, ["a.ts", "leaf.ts", "shared.ts"]);
    assert_eq!(visits, 3);
}
