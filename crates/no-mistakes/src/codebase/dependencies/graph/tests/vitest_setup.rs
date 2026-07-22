#[test]
fn vitest_setup_edges_are_lazy_and_connect_setup_dependencies_to_tests() {
    let setup = p("/repo/test/setup.ts");
    let helper = p("/repo/test/setup-helper.ts");
    let test = p("/repo/src/widget.test.ts");
    let graph = from_raw_maps(
        p("/repo"),
        raw_fwd(&[
            ("/repo/test/setup.ts", &["/repo/test/setup-helper.ts"]),
            ("/repo/src/widget.test.ts", &[]),
        ]),
        raw_rev(&[("/repo/test/setup-helper.ts", &["/repo/test/setup.ts"])]),
    )
    .with_vitest_setup_projects(vec![VitestSetupProject {
        config: Some("vitest.config.ts".to_string()),
        scope: Some(".".to_string()),
        filter: crate::codebase::test_discovery::ProjectTestFilter::from_project_ref(
            &crate::integration_tests::types::ConfigProject {
                config: Some("vitest.config.ts".to_string()),
                policy_name: None,
                runner_project_arg: None,
                scope: Some(".".to_string()),
                include: vec!["src/**/*.test.ts".to_string()],
                exclude: Vec::new(),
                vitest_setup: Vec::new(),
            },
        )
        .unwrap(),
        setups: vec![(setup.clone(), VitestSetupField::SetupFiles)],
    }]);

    assert!(!graph.vitest_setup_edges_materialized());
    assert_eq!(
        graph.dependents_of_node(&NodeId::File(setup.clone())),
        Some(&vec![(
            NodeId::File(test.clone()),
            EdgeKind::VitestSetup(VitestSetupField::SetupFiles),
        )]),
    );
    assert!(graph.vitest_setup_edges_materialized());

    let impacted = graph.dependents_of(&[NodeId::File(helper)], None, None);
    assert!(
        impacted
            .iter()
            .any(|entry| entry.node == NodeId::File(test.clone()))
    );
}

#[test]
fn vitest_setup_edge_detail_and_sort_key_are_stable() {
    let setup = EdgeKind::VitestSetup(VitestSetupField::GlobalSetup);
    assert_eq!(setup.as_str(), "vitest-setup");
    assert_eq!(setup.detail(), Some("globalSetup"));
    assert_eq!(setup.sort_key(), (36, 1));
}
