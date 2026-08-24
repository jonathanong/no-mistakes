use super::*;

fn package_plan(path: &str, fallback: bool) -> serde_json::Value {
    let root = fixture("tests-impact");
    let mut args = vec![
        "tests",
        "plan",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        path,
    ];
    if fallback {
        args.extend(["--global-config-fallback", "true"]);
    }
    args.push("--json");
    let output = run(&args);
    assert!(output.status.success());
    serde_json::from_str(&stdout(&output)).unwrap()
}

fn has_no_baseline_warning(plan: &serde_json::Value) -> bool {
    plan["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["type"] == "package-manifest-no-baseline")
}

#[test]
fn package_json_without_baseline_warns_without_default_fallback() {
    let plan = package_plan("package.json", false);
    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["fallback_reason"].is_null());
    assert!(plan["selected_tests"].as_array().unwrap().is_empty());
    assert!(has_no_baseline_warning(&plan));
}

#[test]
fn package_json_without_baseline_can_opt_into_global_fallback() {
    let plan = package_plan("package.json", true);
    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("Could not determine old content"));
    assert!(has_no_baseline_warning(&plan));
    let selected = plan["selected_tests"].as_array().unwrap();
    assert_eq!(selected.len(), 2);
    let mut names: Vec<&str> = selected
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    names.sort_unstable();
    assert_eq!(names, vec!["a.test.mts", "dynamic.test.mts"]);
    assert!(selected.iter().all(|test| test["confidence"] == "high"));
}

#[test]
fn unregistered_nested_package_json_is_ignored() {
    let plan = package_plan("nested/package.json", false);
    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["warnings"].as_array().unwrap().is_empty());
    assert!(plan["selected_tests"].as_array().unwrap().is_empty());
}
