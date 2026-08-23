#[test]
fn flow_prepared_graph_honors_explicit_config_without_nested_discovery() {
    let root = fixture_root("graph-default-route-config");
    let empty_config = fixture_root("graph-empty-route-config").join(".no-mistakes.yml");
    let options = |config| FlowOptions {
        target: "src/client.ts".to_string(),
        root: root.clone(),
        tsconfig: None,
        config,
        direction: FlowDirection::Deps,
        depth: 1,
        relationships: vec![RelationshipArg::Route],
    };

    let default = run(&options(None)).unwrap();
    assert!(default
        .nodes
        .iter()
        .any(|node| node.file.as_deref() == Some("backend/api/users.mts")));
    let explicit = run(&options(Some(empty_config))).unwrap();
    assert!(!explicit
        .nodes
        .iter()
        .any(|node| node.file.as_deref() == Some("backend/api/users.mts")));

    let source = include_str!("../flow_query.rs");
    let run_body = source
        .split("pub fn run(options: &FlowOptions)")
        .nth(1)
        .and_then(|source| source.split("include!(\"flow_query_traverse.rs\")").next())
        .expect("flow run body");
    assert_eq!(
        run_body.matches("VisiblePathSnapshot::new(&root)").count(),
        1
    );
    assert_eq!(run_body.matches("load_v2_config_from_visible(").count(), 1);
    assert_eq!(run_body.matches("config_from_loaded_v2(").count(), 1);
    assert_eq!(run_body.matches("prepare_graph_config(").count(), 1);
    assert_eq!(
        run_body
            .matches("build_with_plan_files_prepared_config_facts_and_resolution_cache(")
            .count(),
        1
    );
    assert!(!run_body.contains("build_with_plan_and_files_config("));
}
