#[test]
fn split_config_revision_changes_only_invalidate_the_affected_runner() {
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/test-plan/config-fragments");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path();
    crate::test_support::git_init(root);
    crate::test_support::git_commit_all(root, "split config base");
    std::fs::copy(
        root.join("configs/tests-change.yml"),
        root.join(".no-mistakes/tests/plan.yml"),
    )
    .unwrap();
    crate::test_support::git_commit_all(root, "vitest fragment change");
    for (framework, expected) in [("vitest", true), ("playwright", false)] {
        let output = tests_plan_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root, "framework": framework, "base": "HEAD~1", "head": "HEAD",
                "globalConfigFallback": true
            })
            .to_string(),
        ))
        .unwrap();
        let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(plan["fallback_triggered"], expected, "{framework}: {plan}");
    }
}

#[test]
fn split_config_diff_and_manual_fragment_changes_preserve_conservative_fallback() {
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/test-plan/config-fragments");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path();
    crate::test_support::git_init(root);
    crate::test_support::git_commit_all(root, "split config base");
    std::fs::copy(
        root.join("configs/tests-change.yml"),
        root.join(".no-mistakes/tests/plan.yml"),
    )
    .unwrap();
    let diff = std::process::Command::new("git")
        .args(["diff"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(diff.status.success());
    for (use_diff, expected) in [(true, false), (false, true)] {
        let mut args = json!({"root": root, "framework": "playwright",
            "changedFiles": [".no-mistakes/tests/plan.yml"], "globalConfigFallback": true});
        if use_diff {
            args["diff"] = String::from_utf8(diff.stdout.clone()).unwrap().into();
        }
        let output =
            tests_plan_json_impl(crate::napi_api::options::test_json_arg(args.to_string()))
                .unwrap();
        let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(plan["fallback_triggered"], expected, "{plan}");
    }
}

#[test]
fn deleted_and_renamed_fragments_are_compared_at_both_git_endpoints() {
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/test-plan/config-fragments");
    for rename in [false, true] {
        let fixture = crate::test_support::materialize_saved_fixture(&source);
        let root = fixture.path();
        crate::test_support::git_init(root);
        crate::test_support::git_commit_all(root, "split config base");
        let fragment = root.join(".no-mistakes/tests/plan.yml");
        if rename {
            std::fs::rename(fragment, root.join(".no-mistakes/tests/renamed.yaml")).unwrap();
        } else {
            std::fs::remove_file(fragment).unwrap();
        }
        crate::test_support::git_commit_all(root, "fragment endpoint change");
        let output = tests_plan_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root, "framework": "playwright", "base": "HEAD~1", "head": "HEAD",
                "globalConfigFallback": true
            })
            .to_string(),
        ))
        .unwrap();
        let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            plan["fallback_triggered"], !rename,
            "rename={rename}: {plan}"
        );
    }
}
