use super::*;
use std::collections::BTreeSet;

#[test]
fn vitest_config_parser_covers_root_and_nested_projects() {
    let root = fixture("coverage");
    let expected_errors = BTreeSet::from([
        "vitest.empty-array-invalid.mts",
        "vitest.invalid.mts",
        "vitest.invalid-project.mts",
        "vitest.project-exclude-invalid.mts",
    ]);
    let mut policy_names = BTreeSet::new();

    for file in coverage_files("vitest.", ".mts") {
        let path = root.join(&file);
        let source = std::fs::read_to_string(&path).unwrap();
        let result = parse_vitest_fixture(&source, &path, &root);
        if expected_errors.contains(file.as_str()) {
            assert!(result.is_err(), "expected {file} to be rejected");
            continue;
        }
        for project in result.unwrap_or_else(|error| panic!("{file} should parse: {error:#}")) {
            if let Some(policy_name) = project.policy_name {
                policy_names.insert(policy_name);
            }
        }
    }

    for expected in [
        "root-vitest",
        "nested",
        "vitest-root-call-import",
        "vitest-object-call-destructure-body",
        "vitest-member-spread-named",
        "vitest-test-sourced-reexport",
    ] {
        assert!(
            policy_names.contains(expected),
            "missing Vitest policy {expected}"
        );
    }
    assert!(!policy_names.contains("vitest-root-spread-missing"));
}

fn saved_fixture(name: &str) -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config")
        .join(name);
    crate::test_support::materialize_saved_fixture(&source)
}

#[test]
fn vitest_setup_does_not_resolve_a_declaration_file_as_runtime() {
    let fixture = saved_fixture("vitest-declaration-only");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let setup = &parse_vitest_fixture(&source, &path, &root).unwrap()[0].vitest_setup[0];

    assert_eq!(setup.specifier.as_deref(), Some("./declaration-only"));
    assert!(setup.resolved_path.is_none());
    assert!(!setup.trigger_paths.iter().any(|path| {
        path.file_name()
            .is_some_and(|name| name == "declaration-only.d.ts")
    }));
}

fn assert_workspace_project(name: &str, project: &str) {
    let fixture = saved_fixture(name);
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.workspace.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let projects = parse_vitest_fixture(&source, &path, &root).unwrap();
    assert_eq!(projects[0].policy_name.as_deref(), Some(project));
    assert_eq!(
        projects[0].vitest_setup[0].resolved_path.as_deref(),
        Some(root.join("workspace-setup.ts").as_path())
    );
}

#[test]
fn vitest_workspace_default_array_keeps_project_setup_ownership() {
    assert_workspace_project("vitest-workspace-default", "workspace-project");
}

#[test]
fn vitest_workspace_direct_default_array_is_parsed() {
    assert_workspace_project("vitest-workspace-direct-array", "direct-array-project");
}

#[test]
fn vitest_workspace_named_export_reexported_as_default_is_parsed() {
    assert_workspace_project("vitest-workspace-named-reexport", "named-reexport-project");
}

#[test]
fn arbitrary_default_call_is_not_a_workspace_config() {
    let fixture = saved_fixture("vitest-workspace-default");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("not-workspace.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();

    assert!(parse_vitest_fixture(&source, &path, &root)
        .unwrap()
        .is_empty());
}

#[test]
fn vitest_project_glob_accepts_config_suffixes() {
    let fixture = saved_fixture("vitest-config-suffix-glob");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let visible = crate::codebase::ts_source::discover_visible_paths(&root)
        .into_iter()
        .collect();
    let tsconfig = test_support::tsconfig_without_config(&root);
    let projects =
        test_support::parse_vitest_from_visible(&source, &path, &root, &root, &tsconfig, &visible)
            .unwrap();

    assert_eq!(
        projects
            .iter()
            .filter_map(|project| project.policy_name.as_deref())
            .collect::<Vec<_>>(),
        ["e2e-suffix", "unit-suffix"]
    );
}
