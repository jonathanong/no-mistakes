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
    let targets = &report.tests[0].targets;
    assert_eq!(targets.len(), 3);
    let target = targets
        .iter()
        .find(|target| target.project.as_deref() == Some("inline-object"))
        .unwrap();
    assert!(targets
        .iter()
        .any(|target| target.project.as_deref() == Some("nested-object")));
    assert!(targets.iter().any(|target| target.project.is_none()));
    assert!(target.workspace);
    assert_eq!(target.config.as_deref(), Some("vitest.projects.ts"));
    assert_eq!(
        target_command(target),
        [
            "vitest",
            "--workspace",
            "vitest.projects.ts",
            "--project",
            "inline-object",
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

#[test]
fn cargo_targets_and_plan_warn_without_inventing_ambiguous_commands() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/cargo-target-ownership"),
    );
    let report = generate_targets(&TargetsArgs {
        framework: TestFramework::Cargo,
        files: vec![
            "crates/custom/library/tests.rs".into(),
            "crates/engine/src/tests.rs".into(),
            "crates/custom/harness/disabled.rs".into(),
            "orphan/tests/example.rs".into(),
        ],
        root,
        config: None,
        format: None,
        json: false,
    })
    .unwrap();
    assert_eq!(report.tests.len(), 4);
    assert_eq!(report.warnings.len(), 3);
    assert!(report
        .warnings
        .iter()
        .all(|warning| warning.r#type == "cargo-target-unresolved"));
    let mut plan = TestPlan {
        selected_tests: report
            .tests
            .into_iter()
            .map(|row| crate::tests::SelectedTest {
                test_file: row.test_file,
                confidence: crate::tests::Confidence::High,
                reasons: Vec::new(),
                targets: row.targets,
            })
            .collect(),
        ..Default::default()
    };
    plan.finish(false, &[]);
    plan.finish(false, &[]);
    assert_eq!(
        plan.warnings.len(),
        3,
        "finishing again must not duplicate warnings"
    );
    assert_eq!(plan.execution_targets.len(), 1);
    assert!(plan.execution_targets[0]
        .runner_args
        .ends_with(&["--lib".into()]));
    assert!(commands_for_plan(&plan)
        .iter()
        .all(|command| command.contains("-p renamed-package --lib")));
}
