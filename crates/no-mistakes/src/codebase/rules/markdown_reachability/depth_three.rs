use super::*;

// Output cases moved to cli_mrf_snapshot_markdown_reachability. Keep bounded
// index traversal that check JSON cannot distinguish.

#[test]
fn bounded_index_traversal_handles_cycles_duplicate_links_and_multiple_roots() {
    let path = |name: &str| PathBuf::from(name);
    let graph = BTreeMap::from([
        (
            path("AGENTS.md"),
            vec![path("README.md"), path("README.md")],
        ),
        (
            path("README.md"),
            vec![path("README.md"), path("domain/README.md")],
        ),
        (
            path("domain/README.md"),
            vec![path("README.md"), path("guide.md")],
        ),
        (path("other/AGENTS.md"), vec![path("direct.md")]),
        (path("guide.md"), vec![path("forbidden.md")]),
        (path("direct.md"), vec![]),
        (path("forbidden.md"), vec![]),
    ]);
    let roots = BTreeSet::from(["AGENTS.md".to_string()]);
    let indexes = BTreeSet::from(["README.md".to_string()]);
    let reachable = graph::index_reachable(&roots, &indexes, &graph, 3);
    assert!(reachable.contains(Path::new("guide.md")));
    assert!(reachable.contains(Path::new("direct.md")));
    assert!(!reachable.contains(Path::new("forbidden.md")));
    assert!(!graph::index_reachable(&roots, &indexes, &graph, 2).contains(Path::new("guide.md")));
    assert!(
        !graph::index_reachable(&BTreeSet::new(), &indexes, &graph, 3)
            .contains(Path::new("guide.md"))
    );
}
