use super::*;

fn projects_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-config/vitest-projects-target"),
    )
}

fn json_workspace_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-config/vitest-workspace-json"),
    )
}

#[test]
fn vitest_projects_target_command_uses_workspace_flag() {
    let report = generate_targets(&TargetsArgs {
        framework: TestFramework::Vitest,
        files: vec![PathBuf::from("tests/unit.test.ts")],
        root: projects_fixture(),
        config: None,
        format: None,
        json: false,
    })
    .unwrap();
    let target = &report.tests[0].targets[0];

    assert!(target.workspace);
    assert_eq!(target.config.as_deref(), Some("vitest.projects.ts"));
    assert_eq!(
        target_command(target),
        [
            "vitest",
            "--workspace",
            "vitest.projects.ts",
            "--project",
            "unit",
            "tests/unit.test.ts",
        ]
    );
}

#[test]
fn vitest_json_project_object_name_uses_label_in_target_command() {
    let report = generate_targets(&TargetsArgs {
        framework: TestFramework::Vitest,
        files: vec![PathBuf::from("inline/inline.test.ts")],
        root: json_workspace_fixture(),
        config: None,
        format: None,
        json: false,
    })
    .unwrap();
    let target = report
        .tests
        .iter()
        .flat_map(|test| &test.targets)
        .find(|target| target.project.as_deref() == Some("json-inline"))
        .expect("object-form JSON project target");

    assert_eq!(target.project.as_deref(), Some("json-inline"));
    assert_eq!(
        target_command(target),
        [
            "vitest",
            "--workspace",
            "vitest.workspace.json",
            "--project",
            "json-inline",
            "inline/inline.test.ts",
        ]
    );
}
