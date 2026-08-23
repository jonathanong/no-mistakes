use super::*;

#[test]
fn flow_query_covers_deps_and_dependents_directions() {
    let root = fixture_root("tests-impact-symbol");
    for direction in [FlowDirection::Deps, FlowDirection::Dependents] {
        let report = run(&FlowOptions {
            target: "utils.mts#parseDate".to_string(),
            root: root.clone(),
            tsconfig: None,
            config: None,
            direction,
            depth: 1,
            relationships: vec![RelationshipArg::Import],
        })
        .unwrap();

        assert_eq!(report.target, "utils.mts#parseDate");
        assert!(report
            .nodes
            .iter()
            .any(|node| node.id == "utils.mts#parseDate"));
    }
}

#[test]
fn flow_query_symbol_dependents_skip_owner_file_bridge() {
    let root = fixture_root("tests-impact-symbol");
    let report = run(&FlowOptions {
        target: "utils.mts#parseDate".to_string(),
        root,
        tsconfig: None,
        config: None,
        direction: FlowDirection::Dependents,
        depth: 1,
        relationships: vec![RelationshipArg::Import],
    })
    .unwrap();

    assert!(report
        .nodes
        .iter()
        .any(|node| node.id == "utils.mts#parseDate"));
    assert!(!report.nodes.iter().any(|node| node.id == "utils.mts"));
}

#[test]
fn flow_ignores_automatic_ignored_tsconfig_but_honors_explicit_path() {
    let fixture = gitignore_fixture();
    let options = |tsconfig| FlowOptions {
        target: "entry.ts".to_string(),
        root: fixture.path().to_path_buf(),
        tsconfig,
        config: None,
        direction: FlowDirection::Deps,
        depth: 1,
        relationships: vec![RelationshipArg::Import],
    };

    let automatic = run(&options(None)).unwrap();
    assert!(automatic
        .nodes
        .iter()
        .any(|node| { node.kind == "module" && node.module.as_deref() == Some("@lib/forbidden") }));
    assert!(!automatic
        .nodes
        .iter()
        .any(|node| node.file.as_deref() == Some("src/forbidden.ts")));

    let explicit = run(&options(Some(PathBuf::from("tsconfig.json")))).unwrap();
    assert!(explicit
        .nodes
        .iter()
        .any(|node| node.file.as_deref() == Some("src/forbidden.ts")));
}
