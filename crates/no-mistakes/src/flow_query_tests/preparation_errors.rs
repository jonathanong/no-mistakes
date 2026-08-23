use super::*;

#[test]
fn flow_query_explicit_missing_tsconfig_errors() {
    let root = fixture_root("simple");
    let error = resolve_tsconfig(&root, Some(Path::new("missing.tsconfig.json"))).unwrap_err();

    assert!(error.to_string().contains("missing.tsconfig.json"));

    let error = run(&FlowOptions {
        target: "a.mts".to_string(),
        root,
        tsconfig: Some(PathBuf::from("missing.tsconfig.json")),
        config: None,
        direction: FlowDirection::Deps,
        depth: 1,
        relationships: vec![RelationshipArg::Import],
    })
    .unwrap_err();
    assert!(error.to_string().contains("missing.tsconfig.json"));
}
