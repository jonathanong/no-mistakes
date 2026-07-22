use super::*;

#[test]
fn vitest_inline_setup_inheritance_requires_extends_true() {
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/test-config/vitest-extends");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = parse_vitest_fixture(&source, &path, &root).unwrap();

    for name in ["default", "false", "nonboolean", "spread-false-last"] {
        assert!(projects
            .iter()
            .find(|project| project.policy_name.as_deref() == Some(name))
            .unwrap()
            .vitest_setup
            .is_empty());
    }
    for name in ["true", "spread-true-last"] {
        let inherited = projects
            .iter()
            .find(|project| project.policy_name.as_deref() == Some(name))
            .unwrap();
        assert_eq!(inherited.vitest_setup.len(), 2, "{name}");
        assert_eq!(
            inherited
                .vitest_setup
                .iter()
                .map(|setup| setup.field.as_str())
                .collect::<Vec<_>>(),
            vec!["setupFiles", "globalSetup"],
            "{name}",
        );
    }
    let merged = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("merged-setups"))
        .unwrap();
    assert_eq!(
        merged
            .vitest_setup
            .iter()
            .filter(|setup| setup.field.as_str() == "setupFiles")
            .map(|setup| setup.specifier.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["./root-setup.ts", "./project-setup.ts"],
    );
    assert_merged_provenance(merged, "./root-setup.ts");
    assert_eq!(
        merged
            .vitest_setup
            .iter()
            .filter(|setup| setup.field.as_str() == "globalSetup")
            .map(|setup| setup.specifier.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["./root-global.ts", "./project-global.ts"],
    );
    assert_merged_provenance(merged, "./root-global.ts");
    let standalone = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("standalone"))
        .unwrap();
    assert_eq!(standalone.vitest_setup.len(), 1);
    assert_eq!(
        standalone.vitest_setup[0].specifier.as_deref(),
        Some("./standalone-setup.ts")
    );
}

fn assert_merged_provenance(
    project: &crate::integration_tests::types::ConfigProject,
    specifier: &str,
) {
    let setup = project
        .vitest_setup
        .iter()
        .find(|setup| setup.specifier.as_deref() == Some(specifier))
        .unwrap();
    assert!(setup
        .trigger_paths
        .iter()
        .any(|path| path.ends_with("setup-values.ts")));
    assert!(setup
        .trigger_paths
        .iter()
        .any(|path| path.ends_with("vitest.config.ts")));
}
