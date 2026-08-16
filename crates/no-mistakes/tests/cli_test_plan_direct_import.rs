mod common;

use common::{fixture, run, stdout};

fn selected_files(plan: &serde_json::Value) -> Vec<&str> {
    plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect()
}

fn group_selected<'a>(plan: &'a serde_json::Value, group_type: &str) -> Vec<&'a str> {
    plan["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["type"] == group_type)
        .unwrap()["selected"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| path.as_str().unwrap())
        .collect()
}

#[test]
fn limited_plan_keeps_same_directory_direct_importer() {
    let root = fixture("test-plan-direct-import-limit");
    let output = run(&[
        "tests",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "src/dev-server.mts",
        "--environment",
        "prePush",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    assert_eq!(selected_files(&plan), ["src/dev-server.test.mts"]);
    assert_eq!(group_selected(&plan, "direct"), ["src/dev-server.test.mts"]);
    assert!(group_selected(&plan, "dependencies").is_empty());
}

#[test]
fn unlimited_plan_puts_one_hop_imports_in_direct_not_two_hop() {
    let root = fixture("test-plan-direct-import-limit");
    let output = run(&[
        "tests",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "src/dev-server.mts",
        "--environment",
        "full",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    assert_eq!(
        group_selected(&plan, "direct"),
        ["src/dev-server.test.mts", "tests/dev-server.test.mts"]
    );
    let dependencies = group_selected(&plan, "dependencies");
    assert!(dependencies.contains(&"src/mid.test.mts"));
    assert!(dependencies.contains(&"aaa-01.test.mts"));
    assert!(!dependencies.contains(&"src/dev-server.test.mts"));
    assert!(!group_selected(&plan, "direct").contains(&"src/mid.test.mts"));
}

#[test]
fn limited_plan_prefers_self_selected_changed_test_over_earlier_importer() {
    let root = fixture("test-plan-direct-import-limit");
    let output = run(&[
        "tests",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "src/dev-server.mts",
        "--changed-file",
        "zzz.test.mts",
        "--environment",
        "prePush",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(selected_files(&plan), ["zzz.test.mts"]);
    assert_eq!(group_selected(&plan, "direct"), ["zzz.test.mts"]);
}

#[test]
fn unlimited_plan_does_not_duplicate_changed_test_that_also_imports_source() {
    let root = fixture("test-plan-direct-import-limit");
    let output = run(&[
        "tests",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "src/dev-server.mts",
        "--changed-file",
        "src/dev-server.test.mts",
        "--environment",
        "full",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let direct = group_selected(&plan, "direct");
    assert_eq!(
        direct
            .iter()
            .filter(|path| **path == "src/dev-server.test.mts")
            .count(),
        1
    );
}

#[test]
fn sampled_direct_limit_samples_over_budget_changed_tests() {
    let root = fixture("test-plan-direct-import-limit");
    let output = run(&[
        "tests",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "yyy.test.mts",
        "--changed-file",
        "zzz.test.mts",
        "--environment",
        "sampledDirect",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    // first_take would keep yyy.test.mts; stable_take keeps zzz.test.mts.
    assert_eq!(selected_files(&plan), ["zzz.test.mts"]);
}

#[test]
fn sampled_direct_limit_still_keeps_self_selected_changed_test() {
    let root = fixture("test-plan-direct-import-limit");
    let output = run(&[
        "tests",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "src/dev-server.mts",
        "--changed-file",
        "zzz.test.mts",
        "--environment",
        "sampledDirect",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(selected_files(&plan), ["zzz.test.mts"]);
    assert_eq!(group_selected(&plan, "direct"), ["zzz.test.mts"]);
}

#[test]
fn changed_test_file_stays_self_selected_in_direct() {
    let root = fixture("test-plan-direct-import-limit");
    let output = run(&[
        "tests",
        "plan",
        "vitest",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "src/dev-server.test.mts",
        "--environment",
        "prePush",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(selected_files(&plan), ["src/dev-server.test.mts"]);
    assert_eq!(group_selected(&plan, "direct"), ["src/dev-server.test.mts"]);
    assert_eq!(
        plan["selected_tests"][0]["reasons"][0]["via"],
        serde_json::json!(["self"])
    );
}
