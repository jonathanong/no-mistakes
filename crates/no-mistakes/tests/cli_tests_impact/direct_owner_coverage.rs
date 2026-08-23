use super::*;

#[test]
fn tests_plan_direct_test_owner_keeps_a_changed_owned_test_as_self() {
    let root = fixture("test-plan-config");
    let output = run(&[
        "tests",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "source.test.mts",
        "--direct-test-owner",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    assert_eq!(plan["selected_tests"][0]["test_file"], "source.test.mts");
    assert_eq!(
        plan["selected_tests"][0]["reasons"][0],
        serde_json::json!({
            "changed_file": "source.test.mts",
            "path": ["source.test.mts"],
            "via": ["self"],
        })
    );
    assert_eq!(plan["selected_tests"][0]["targets"][0]["runner"], "vitest");
}

#[test]
fn tests_plan_direct_test_owner_requires_framework_and_rejects_limits() {
    let root = fixture("test-plan-config");
    let missing_framework = run(&[
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--direct-test-owner",
    ]);
    assert!(!missing_framework.status.success());
    let missing_framework = String::from_utf8_lossy(&missing_framework.stderr);
    assert!(
        missing_framework.contains("--direct-test-owner requires a framework"),
        "{missing_framework}"
    );
    assert!(
        missing_framework.contains("tests plan vitest --direct-test-owner"),
        "{missing_framework}"
    );
    assert!(
        missing_framework.contains("framework-specific test ownership"),
        "{missing_framework}"
    );

    let limit = run(&[
        "tests",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--direct-test-owner",
        "--limit-files",
        "1",
    ]);
    assert!(!limit.status.success());
    let stderr = String::from_utf8_lossy(&limit.stderr);
    assert!(
        stderr.contains("--direct-test-owner conflicts with --limit-percent, --limit-files, and --global-config-fallback"),
        "{stderr}"
    );
    assert!(
        stderr.contains("remove those policy overrides because direct ownership bypasses configured plan policy"),
        "{stderr}"
    );

    let entrypoint = run(&[
        "tests",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "source.ts",
        "--direct-test-owner",
        "--entrypoint",
        "source.ts",
    ]);
    assert!(!entrypoint.status.success());
    let stderr = String::from_utf8_lossy(&entrypoint.stderr);
    assert!(
        stderr.contains("--direct-test-owner conflicts with --entrypoint"),
        "{stderr}"
    );
    assert!(stderr.contains("tests impact"), "{stderr}");
}
