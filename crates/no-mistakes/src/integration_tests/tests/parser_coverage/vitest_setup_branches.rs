use super::*;

#[test]
fn vitest_setup_collects_static_conditional_branches() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-setup-dependencies"),
    );
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = parse_vitest_fixture(&source, &path, &root).unwrap();
    let setup = &projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("conditional-setup"))
        .expect("conditional project")
        .vitest_setup;

    assert_eq!(setup.len(), 2, "{setup:#?}");
    assert_eq!(
        setup
            .iter()
            .map(|dependency| dependency.specifier.as_deref())
            .collect::<Vec<_>>(),
        [
            Some("../setup/conditional-a.ts"),
            Some("../setup/conditional-b.ts")
        ]
    );
    assert!(setup.iter().all(|dependency| dependency
        .trigger_paths
        .contains(&root.join("config/branch-selector.ts"))));
    assert_eq!(
        setup[0].resolved_path.as_deref(),
        Some(root.join("setup/conditional-a.ts").as_path())
    );
    assert_eq!(
        setup[1].resolved_path.as_deref(),
        Some(root.join("setup/conditional-b.ts").as_path())
    );
}

#[test]
fn vitest_setup_branch_expansion_has_a_total_budget() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-config/vitest-setup-bounds"),
    );
    let path = root.join("packages/foo/vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let tsconfig = test_support::tsconfig_without_config(&root);
    let projects =
        test_support::parse_vitest(&source, &path, path.parent().unwrap(), &root, &tsconfig)
            .unwrap();
    let setup = &projects[0].vitest_setup;

    assert_eq!(setup.len(), 1, "{setup:#?}");
    assert_eq!(setup[0].specifier, None);
    assert!(setup[0].trigger_paths.contains(&path));
    assert!(setup[0].trigger_paths.contains(&root.join("packages/foo")));
    assert!(setup[0]
        .trigger_paths
        .contains(&root.join("packages/foo/shared/outside.ts")));
    assert_eq!(setup[0].resolution_base, root.join("packages/foo"));

    let rebased = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("bounded-rebased"))
        .expect("bounded imported helper project");
    assert_eq!(rebased.vitest_setup.len(), 1, "{rebased:#?}");
    assert!(rebased.vitest_setup[0]
        .conservative_specifiers
        .contains("../shared/outside.ts"));
    assert!(rebased.vitest_setup[0]
        .trigger_paths
        .contains(&root.join("packages/shared/outside.ts")));
}

#[test]
fn vitest_setup_branch_expansion_has_a_depth_limit() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-config/vitest-setup-depth"),
    );
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = parse_vitest_fixture(&source, &path, &root).unwrap();

    assert_eq!(projects.len(), 3);
    for project in projects {
        assert_eq!(project.vitest_setup.len(), 1);
        let setup = &project.vitest_setup[0];
        assert_eq!(setup.specifier, None);
        assert!(setup.trigger_paths.contains(&path));
        assert!(setup.trigger_paths.contains(&root));
    }
}
