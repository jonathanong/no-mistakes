use super::*;
use crate::integration_tests::types::Framework;
use std::path::PathBuf;

fn saved_fixture(name: &str) -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config")
        .join(name);
    crate::test_support::materialize_saved_fixture(&source)
}

#[test]
fn json_workspace_projects_accept_boolean_extends() {
    let fixture = saved_fixture("vitest-workspace-json-extends");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let visible = crate::codebase::ts_source::discover_visible_paths(&root);
    let tsconfig = test_support::tsconfig_without_config(&root);
    let projects = crate::integration_tests::project_config::load_projects_from_visible(
        &root,
        Framework::Vitest,
        None,
        &visible,
        &tsconfig,
    )
    .unwrap();

    let project = |name: &str| {
        projects
            .iter()
            .find(|project| project.policy_name.as_deref() == Some(name))
            .unwrap()
    };
    let inherited = project("json-extends-true");
    let independent = project("json-extends-false");

    // JSON arrays describe independent projects; these assertions ensure each
    // boolean extends value preserves that project's own setup declaration.
    assert_eq!(inherited.vitest_setup[0].field.as_str(), "setupFiles");
    assert_eq!(
        inherited.vitest_setup[0].specifier.as_deref(),
        Some("./setup.ts")
    );
    assert_eq!(independent.vitest_setup[0].field.as_str(), "globalSetup");
    assert_eq!(
        independent.vitest_setup[0].specifier.as_deref(),
        Some("./global.ts")
    );
}
