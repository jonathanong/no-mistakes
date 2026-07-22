#[test]
fn tests_plan_json_manual_config_change_does_not_borrow_git_endpoints() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/target-scoped-triggers");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path();
    crate::test_support::git_init(root);
    crate::test_support::git_commit_all(root, "initial fixture");
    std::fs::copy(
        root.join("configs/vitest-change.yml"),
        root.join(".no-mistakes.yml"),
    )
    .unwrap();

    let output = tests_plan_json_impl(
        json!({
            "root": root,
            "framework": "playwright",
            "changedFiles": [".no-mistakes.yml"],
            "base": "HEAD",
            "head": "HEAD",
            "globalConfigFallback": true
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(plan["fallback_triggered"], true, "{plan:?}");
}

#[test]
fn tests_plan_json_overlapping_manual_and_git_config_changes_fail_open() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/target-scoped-triggers");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path();
    crate::test_support::git_init(root);
    crate::test_support::git_commit_all(root, "initial fixture");
    std::fs::copy(
        root.join("configs/vitest-change.yml"),
        root.join(".no-mistakes.yml"),
    )
    .unwrap();
    crate::test_support::git_commit_all(root, "vitest config change");
    std::fs::copy(
        root.join("configs/vitest-and-playwright-change.yml"),
        root.join(".no-mistakes.yml"),
    )
    .unwrap();

    for framework in ["vitest", "playwright"] {
        let output = tests_plan_json_impl(
            json!({
                "root": root,
                "framework": framework,
                "changedFiles": [".no-mistakes.yml"],
                "base": "HEAD~1",
                "head": "HEAD",
                "globalConfigFallback": true
            })
            .to_string(),
        )
        .unwrap();
        let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(plan["fallback_triggered"], true, "{framework}: {plan:?}");
    }
}

#[test]
fn tests_plan_json_manual_config_requires_same_path_diff_endpoint() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/target-scoped-triggers");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path();
    std::fs::rename(
        root.join(".no-mistakes.yml"),
        root.join(".no-mistakes.yaml"),
    )
    .unwrap();
    crate::test_support::git_init(root);
    crate::test_support::git_commit_all(root, "yaml config fixture");
    std::fs::copy(
        root.join("configs/vitest-change.yml"),
        root.join(".no-mistakes.yaml"),
    )
    .unwrap();
    let diff = std::process::Command::new("git")
        .args(["diff", "--", ".no-mistakes.yaml"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(diff.status.success());
    let diff = String::from_utf8(diff.stdout).unwrap();

    for (manual_path, expected) in [(".no-mistakes.yaml", false), (".no-mistakes.yml", true)] {
        let output = tests_plan_json_impl(
            json!({
                "root": root,
                "framework": "playwright",
                "changedFiles": [manual_path],
                "diff": diff,
                "globalConfigFallback": true
            })
            .to_string(),
        )
        .unwrap();
        let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            plan["fallback_triggered"], expected,
            "{manual_path}: {plan:?}"
        );
    }
}
