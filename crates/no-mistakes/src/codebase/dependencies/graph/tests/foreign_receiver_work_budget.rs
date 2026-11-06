use super::*;

#[test]
fn exhausted_foreign_receiver_facts_never_build_a_partial_graph() {
    let root = fixture("foreign-receiver-work-budget");
    let config = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let result = DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    );
    let error = result.err().expect("incomplete graph must fail closed");
    let failure = error
        .downcast_ref::<crate::codebase::check_outcome::CheckInputFailure>()
        .expect("failure retains structured source location");
    assert_eq!(
        failure.file.as_ref().unwrap().file_name().unwrap(),
        "argument-diamond-16.mts"
    );
    assert!(failure.message.contains("per-file work budget"));
    assert!(failure.message.contains("narrower analysis root"));
}

#[test]
fn import_projection_keeps_complete_import_facts_when_value_flow_is_incomplete() {
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("foreign-receiver-work-budget"));
    let source = root.join("src/shared-4096.mts");
    let provider = root.join("src/providers.mts");
    let files = vec![source.clone(), provider.clone()];
    let facts = crate::codebase::ts_source::facts::collect_ts_facts(
        &files,
        TsFactPlan {
            imports: true,
            function_calls: true,
            value_flow: true,
            ..Default::default()
        },
    );
    assert!(facts[&source].value_flow.incomplete_reason.is_some());
    let graph = DepGraph::build_with_plan_files_config_and_facts(
        &root,
        &TsConfig {
            dir: root.clone(),
            paths_dir: root.clone(),
            ..Default::default()
        },
        GraphBuildPlan {
            imports: true,
            ..Default::default()
        },
        &GraphFiles::from_files(files),
        None,
        Some(&facts),
    )
    .unwrap();
    let allowed = HashSet::from([EdgeKind::Import]);
    let dependencies = graph.deps_of(&[NodeId::file(source)], None, Some(&allowed));
    assert!(dependencies
        .iter()
        .any(|entry| entry.node.as_file() == Some(provider.as_path())));
}
