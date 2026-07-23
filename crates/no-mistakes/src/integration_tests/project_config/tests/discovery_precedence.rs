use super::*;
use crate::config::v2::schema::StringOrList;
use std::path::PathBuf;

fn saved_fixture(name: &str) -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config")
        .join(name);
    crate::test_support::materialize_saved_fixture(&source)
}

#[test]
fn default_vitest_workspace_discovery_suppresses_root_config_projects() {
    let fixture = saved_fixture("vitest-workspace-precedence");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let projects = load_projects(&root, Framework::Vitest, None).unwrap();

    assert_eq!(projects.len(), 1);
    let project = &projects[0];
    assert_eq!(project.policy_name.as_deref(), Some("workspace"));
    assert_eq!(project.config.as_deref(), Some("vitest.workspace.ts"));
    assert!(project.workspace);
    assert_eq!(project.vitest_setup.len(), 1);
    assert_eq!(
        project.vitest_setup[0].specifier.as_deref(),
        Some("./workspace-setup.ts")
    );

    let configs = StringOrList::One("vitest.config.ts".to_string());
    let explicit = load_projects(&root, Framework::Vitest, Some(&configs)).unwrap();
    assert_eq!(explicit.len(), 1);
    assert_eq!(explicit[0].policy_name.as_deref(), Some("root"));
    assert!(!explicit[0].workspace);
}

#[test]
fn vitest_workspace_can_explicitly_list_the_root_config() {
    let fixture = saved_fixture("vitest-workspace-listed-root");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let projects = load_projects(&root, Framework::Vitest, None).unwrap();

    assert_eq!(
        projects
            .iter()
            .map(|project| (project.policy_name.as_deref(), project.config.as_deref()))
            .collect::<Vec<_>>(),
        vec![
            (Some("workspace"), Some("vitest.workspace.ts")),
            (Some("root"), Some("vitest.workspace.ts")),
        ]
    );
}
