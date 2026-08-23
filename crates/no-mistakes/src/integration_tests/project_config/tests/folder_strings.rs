use super::*;

#[test]
fn exact_folder_project_strings_do_not_resolve_an_index_module_as_config() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config/vitest-folder-project-index");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());

    let projects = load_projects(&root, Framework::Vitest, None).unwrap();

    assert_eq!(projects.len(), 1, "{projects:#?}");
    assert_eq!(projects[0].scope.as_deref(), Some("project-folder"));
    assert!(projects[0].policy_name.is_none(), "{projects:#?}");
    assert!(projects[0].vitest_setup.is_empty(), "{projects:#?}");
}

#[test]
fn standalone_vitest_project_uses_the_top_level_config_target() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/vitest-setup-dependencies");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let projects = load_projects(&root, Framework::Vitest, None).unwrap();
    let standalone = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("standalone-imported-setup"))
        .unwrap();

    assert_eq!(standalone.config.as_deref(), Some("vitest.config.ts"));
}
