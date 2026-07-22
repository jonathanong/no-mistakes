use super::*;

fn projects_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-config/vitest-projects-target"),
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
