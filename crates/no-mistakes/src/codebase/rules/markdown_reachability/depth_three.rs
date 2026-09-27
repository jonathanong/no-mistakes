use super::*;

const FILES: &[&str] = &[
    "AGENTS.md",
    "README.md",
    "domain/README.md",
    "domain/guide.md",
    "domain/deeper/README.md",
    "domain/deeper/deep.md",
    "shortcut.md",
    "overview.md",
    "first.md",
    "second.md",
    "baseline.json",
];

#[test]
fn three_hops_accepts_only_index_intermediaries_despite_a_shorter_invalid_route() {
    let root = fixture("depth-three");
    // The ordinary shortest route to guide.md uses shortcut.md; a valid
    // longer route through two README indexes must still satisfy depth three.
    let findings = run(
        &root,
        &config(
            "rootFilenames: [AGENTS.md]\nmaxDepth: 3",
            &[
                "domain/guide.md",
                "domain/deeper/deep.md",
                "first.md",
                "second.md",
            ],
            &[],
        ),
        FILES,
    )
    .unwrap();
    assert_eq!(
        findings.iter().map(|f| f.file.as_str()).collect::<Vec<_>>(),
        ["domain/deeper/deep.md", "first.md", "second.md"]
    );
    assert_eq!(
        findings[0].message,
        "reachable only at depth 4; maximum is 3"
    );
    assert_eq!(
        findings[1].message,
        "reachable at depth 2, but an intermediary must be a configured index Markdown file"
    );
    assert_eq!(
        findings[2].message,
        "reachable at depth 3, but an intermediary must be a configured index Markdown file"
    );
}

#[test]
fn two_hops_and_default_depth_do_not_allow_two_index_intermediaries() {
    let root = fixture("depth-three");
    for options in [
        "rootFilenames: [AGENTS.md]",
        "rootFilenames: [AGENTS.md]\nmaxDepth: 2",
    ] {
        let findings = run(&root, &config(options, &["domain/guide.md"], &[]), FILES).unwrap();
        assert_eq!(findings.len(), 1);
        assert!(findings[0]
            .message
            .contains("intermediary must be a configured index"));
    }
}

#[test]
fn third_hop_resolves_baseline_but_fourth_hop_still_matches_exact_depth() {
    let root = fixture("depth-three");
    let findings = run(
        &root,
        &config(
            "rootFilenames: [AGENTS.md]\nmaxDepth: 3\nbaselineFile: baseline.json",
            &["domain/guide.md", "domain/deeper/deep.md"],
            &[],
        ),
        FILES,
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "domain/guide.md");
    assert_eq!(
        findings[0].message,
        "stale baseline entry: is reachable; remove its baseline entry"
    );
}

#[test]
fn configured_index_names_control_each_intermediary() {
    let root = fixture("depth-three");
    let findings = run(
        &root,
        &config(
            "rootFilenames: [AGENTS.md]\nindexFilenames: [shortcut.md]\nmaxDepth: 3",
            &["domain/guide.md", "first.md", "second.md"],
            &[],
        ),
        FILES,
    )
    .unwrap();
    assert_eq!(
        findings.iter().map(|f| f.file.as_str()).collect::<Vec<_>>(),
        ["second.md"]
    );
}

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
