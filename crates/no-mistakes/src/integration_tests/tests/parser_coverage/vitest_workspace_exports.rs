use super::*;
use std::path::PathBuf;

fn saved_fixture(name: &str) -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-config")
        .join(name);
    crate::test_support::materialize_saved_fixture(&source)
}

#[test]
fn vitest_workspace_exports_handle_namespace_reexports_and_safe_empty_forms() {
    for (fixture_name, extension, expected) in [
        (
            "vitest-workspace-namespace",
            "ts",
            Some("namespace-project"),
        ),
        (
            "vitest-workspace-local-and-type",
            "ts",
            Some("local-export-project"),
        ),
        (
            "vitest-workspace-star-fallback",
            "ts",
            Some("star-fallback-project"),
        ),
        ("vitest-workspace-cycle", "ts", None),
        ("vitest-workspace-missing-import", "ts", None),
        ("vitest-workspace-empty-forms", "ts", None),
        ("vitest-workspace-missing-binding", "ts", None),
        ("vitest-workspace-invalid-namespace", "ts", None),
        ("vitest-workspace-import-cycle", "ts", None),
        ("vitest-workspace-default-class", "ts", None),
        ("vitest-workspace-empty-define", "ts", None),
        (
            "vitest-workspace-commonjs-filtering",
            "cjs",
            Some("commonjs-filter-project"),
        ),
    ] {
        let fixture = saved_fixture(fixture_name);
        let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
        let path = root.join(format!("vitest.workspace.{extension}"));
        let source = std::fs::read_to_string(&path).unwrap();
        let projects = parse_vitest_fixture(&source, &path, &root).unwrap();

        assert_eq!(
            projects
                .first()
                .and_then(|project| project.policy_name.as_deref()),
            expected,
            "{fixture_name}",
        );
        assert_eq!(
            projects.len(),
            usize::from(expected.is_some()),
            "{fixture_name}"
        );
    }
}
