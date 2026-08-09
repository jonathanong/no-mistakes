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
