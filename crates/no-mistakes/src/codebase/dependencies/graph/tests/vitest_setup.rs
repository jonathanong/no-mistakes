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

#[test]
fn edge_kind_sort_keys_preserve_resource_order_and_append_vitest() {
    let kinds = [
        EdgeKind::Import,
        EdgeKind::TypeImport,
        EdgeKind::DynamicImport,
        EdgeKind::RouteImport,
        EdgeKind::Require,
        EdgeKind::TestOf,
        EdgeKind::RouteRef,
        EdgeKind::QueueEnqueue,
        EdgeKind::QueueWorker,
        EdgeKind::RouteTest,
        EdgeKind::Layout,
        EdgeKind::MarkdownLink,
        EdgeKind::WorkspaceImport,
        EdgeKind::PackageDependency,
        EdgeKind::CiInvocation,
        EdgeKind::HttpCall,
        EdgeKind::ProcessSpawn,
        EdgeKind::AssetImport,
        EdgeKind::Resource,
        EdgeKind::ReactRender,
        EdgeKind::Selector,
        EdgeKind::SwiftImport,
        EdgeKind::SwiftReference,
        EdgeKind::SwiftPackageDependency,
        EdgeKind::DotnetUsing,
        EdgeKind::DotnetReference,
        EdgeKind::DotnetProjectDependency,
        EdgeKind::TerraformReference,
        EdgeKind::TerraformModuleRef,
        EdgeKind::TerraformOutputRef,
        EdgeKind::WorkflowJob,
        EdgeKind::WorkflowStep,
        EdgeKind::WorkflowNeeds,
        EdgeKind::WorkflowUses,
        EdgeKind::WorkflowRun,
        EdgeKind::WorkflowArtifact,
        EdgeKind::VitestSetup(VitestSetupField::SetupFiles),
        EdgeKind::VitestSetup(VitestSetupField::GlobalSetup),
    ];
    let keys: Vec<_> = kinds.into_iter().map(EdgeKind::sort_key).collect();
    assert_eq!(
        keys,
        (0..=36)
            .map(|key| (key, 0))
            .chain([(36, 1)])
            .collect::<Vec<_>>()
    );
}

#[test]
fn vitest_setup_prefers_nested_owner_without_suppressing_unscoped_owner() {
    let test = p("/repo/src/nested/widget.test.ts");
    let root_setup = p("/repo/setup/root.ts");
    let nested_setup = p("/repo/setup/nested.ts");
    let unscoped_setup = p("/repo/setup/unscoped.ts");
    let graph = from_typed_maps(
        p("/repo"),
        HashMap::from([
            (NodeId::File(test.clone()), Vec::new()),
            (NodeId::Module("vitest".to_string()), Vec::new()),
        ]),
        EdgeMap::new(),
    )
    .with_vitest_setup_projects(vec![
        vitest_project("root", Some("src"), "src/**/*.test.ts", &root_setup),
        vitest_project(
            "nested",
            Some("src/nested"),
            "src/nested/**/*.test.ts",
            &nested_setup,
        ),
        vitest_project(
            "unscoped",
            None,
            "src/nested/**/*.test.ts",
            &unscoped_setup,
        ),
    ]);

    assert_eq!(
        graph.dependencies_of_node(&NodeId::File(test)),
        Some(&vec![
            (
                NodeId::File(nested_setup),
                EdgeKind::VitestSetup(VitestSetupField::SetupFiles),
            ),
            (
                NodeId::File(unscoped_setup),
                EdgeKind::VitestSetup(VitestSetupField::SetupFiles),
            ),
        ]),
    );
}

#[test]
fn vitest_setup_root_scope_is_an_ancestor_of_nested_projects() {
    let test = p("/repo/src/widget.test.ts");
    let root_setup = p("/repo/setup/root.ts");
    let nested_setup = p("/repo/setup/nested.ts");
    let graph = from_typed_maps(
        p("/repo"),
        HashMap::from([
            (NodeId::File(test.clone()), Vec::new()),
            (NodeId::Module("vitest".to_string()), Vec::new()),
        ]),
        EdgeMap::new(),
    )
    .with_vitest_setup_projects(vec![
        vitest_project("root", Some("."), "src/**/*.test.ts", &root_setup),
        vitest_project("nested", Some("src"), "src/**/*.test.ts", &nested_setup),
    ]);

    assert_eq!(
        graph.dependencies_of_node(&NodeId::File(test)),
        Some(&vec![(
            NodeId::File(nested_setup),
            EdgeKind::VitestSetup(VitestSetupField::SetupFiles),
        )]),
    );
}

fn vitest_project(
    config: &str,
    scope: Option<&str>,
    include: &str,
    setup: &Path,
) -> VitestSetupProject {
    let project = crate::integration_tests::types::ConfigProject {
        config: Some(format!("{config}.config.ts")),
        policy_name: None,
        runner_project_arg: None,
        scope: scope.map(str::to_string),
        include: vec![include.to_string()],
        exclude: Vec::new(),
        vitest_setup: Vec::new(),
    };
    VitestSetupProject {
        config: project.config.clone(),
        scope: project.scope.clone(),
        filter: crate::codebase::test_discovery::ProjectTestFilter::from_project_ref(&project)
            .unwrap(),
        setups: vec![(setup.to_path_buf(), VitestSetupField::SetupFiles)],
    }
}
