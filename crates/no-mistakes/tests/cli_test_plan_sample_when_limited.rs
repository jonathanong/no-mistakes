mod common;

use common::{fixture, run, stdout};

#[test]
fn limited_sample_group_defaults_to_first_candidates() {
    let root = fixture("test-plan-sample-when-limited");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "changed.test.mts",
        "--environment",
        "first",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 2);
    assert_eq!(plan["groups"][0]["selected"][0], "changed.test.mts");
    assert_eq!(plan["groups"][1]["selected"][0], "alpha.test.mts");
    assert_eq!(plan["groups"][1]["limit"], 1);
    assert_eq!(plan["groups"][1]["remaining"], 3);
}

#[test]
fn limited_sample_group_can_sample_when_opted_in() {
    let root = fixture("test-plan-sample-when-limited");
    let output = run(&[
        "test",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "changed.test.mts",
        "--environment",
        "sampled",
        "--json",
    ]);

    assert!(output.status.success());
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 2);
    assert_eq!(plan["groups"][0]["selected"][0], "changed.test.mts");
    assert_eq!(plan["groups"][1]["selected"][0], "zeta.test.mts");
    assert_eq!(plan["groups"][1]["limit"], 1);
    assert_eq!(plan["groups"][1]["remaining"], 3);
}
