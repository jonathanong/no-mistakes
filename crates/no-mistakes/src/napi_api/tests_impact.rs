#[test]
fn tests_impact_json_returns_plan_for_file_entrypoint() {
    let root = fixture_root("tests-impact-symbol");
    let options = json!({
        "root": root,
        "entrypoints": ["utils.mts"]
    })
    .to_string();
    let output = tests_impact_json_impl(options).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    let test_files: Vec<&str> = selected
        .iter()
        .map(|t| t["test_file"].as_str().unwrap())
        .collect();
    assert!(test_files.contains(&"service.test.mts"));
    assert!(test_files.contains(&"other.test.mts"));
}

#[test]
fn tests_plan_json_with_diff_content() {
    let root = fixture_root("tests-impact-diff");
    let diff = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase-analysis/tests-impact-diff/sample.diff"),
    )
    .unwrap();
    let options = json!({
        "root": root,
        "diff": diff
    })
    .to_string();
    let output = tests_plan_json_impl(options).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected.iter().any(|t| t["test_file"] == "a.test.mts"),
        "diff should find a.test.mts: {selected:?}"
    );
}

#[test]
fn tests_plan_json_with_entrypoints() {
    let root = fixture_root("tests-impact-diff");
    let options = json!({
        "root": root,
        "entrypoints": ["c.mts"]
    })
    .to_string();
    let output = tests_plan_json_impl(options).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(
        selected.iter().any(|t| t["test_file"] == "a.test.mts"),
        "entrypoint c.mts should find a.test.mts: {selected:?}"
    );
}

#[test]
fn tests_plan_json_without_input_returns_empty() {
    let root = fixture_root("tests-impact-diff");
    let options = json!({ "root": root }).to_string();
    let output = tests_plan_json_impl(options).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    assert!(selected.is_empty());
}
