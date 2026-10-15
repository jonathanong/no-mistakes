use super::*;

#[test]
fn vitest_literal_setup_paths_use_each_effective_project_root() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/vitest-literal-setups"),
    );
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = parse_vitest_fixture(&source, &path, &root).unwrap();
    for (name, expected) in [
        (
            "web",
            vec![
                "web/setup/start.ts",
                "web/setup/explicit.ts",
                "web/setup/global.ts",
            ],
        ),
        ("node", vec!["node/setup/start.ts"]),
    ] {
        let project = projects
            .iter()
            .find(|p| p.policy_name.as_deref() == Some(name))
            .unwrap();
        assert_eq!(
            project
                .vitest_setup
                .iter()
                .map(|s| s.resolved_path.clone().unwrap())
                .collect::<Vec<_>>(),
            expected.iter().map(|p| root.join(p)).collect::<Vec<_>>()
        );
        assert!(project
            .vitest_setup
            .iter()
            .all(|s| s.resolution_base == root.join(name)));
    }
    let missing = &projects
        .iter()
        .find(|p| p.policy_name.as_deref() == Some("missing"))
        .unwrap()
        .vitest_setup[0];
    assert!(missing.resolved_path.is_none());
    assert!(missing
        .resolver_candidate_paths
        .contains(&root.join("missing/setup/deleted.ts")));
    let web = &projects
        .iter()
        .find(|p| p.policy_name.as_deref() == Some("web"))
        .unwrap()
        .vitest_setup[0];
    assert!(web
        .transitive_trigger_paths
        .contains(&root.join("web/setup/helper.ts")));
}
