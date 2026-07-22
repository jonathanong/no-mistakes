fn resource_fixture_root() -> tempfile::TempDir {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/resource-impact"),
    );
    crate::test_support::materialize_saved_fixture(&source)
}

#[test]
fn resource_edges_are_available_through_tests_and_traversal_napi_apis() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();

    let plan = tests_plan_json_impl(
        json!({
            "root": root,
            "changedFiles": ["resources/page.txt"]
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&plan).unwrap();
    assert_eq!(plan["selected_tests"][0]["test_file"], "impact-consumer.test.ts");
    assert_eq!(plan["selected_tests"][0]["reasons"][0]["via"][0], "resource");
    assert_eq!(
        plan["selected_tests"][0]["reasons"][0]["via_details"][0]["type"],
        "resource"
    );

    let impact = tests_impact_json_impl(
        json!({
            "root": root,
            "entrypoints": ["resources/page.txt"]
        })
        .to_string(),
    )
    .unwrap();
    let impact: serde_json::Value = serde_json::from_str(&impact).unwrap();
    assert_eq!(impact["selected_tests"][0]["test_file"], "impact-consumer.test.ts");

    let dependencies = dependencies_json_impl(
        json!({
            "root": root,
            "files": ["impact-consumer.ts"],
            "relationships": ["resource"]
        })
        .to_string(),
    )
    .unwrap();
    let dependencies: serde_json::Value = serde_json::from_str(&dependencies).unwrap();
    assert!(dependencies["files"].as_array().unwrap().iter().any(|file| {
        file["path"] == "resources/page.txt" && file["via"] == json!(["resource"])
    }));

    let dependents = dependents_json_impl(
        json!({
            "root": root,
            "files": ["resources/page.txt"],
            "relationships": ["resource"]
        })
        .to_string(),
    )
    .unwrap();
    let dependents: serde_json::Value = serde_json::from_str(&dependents).unwrap();
    assert!(dependents["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| file["path"] == "impact-consumer.ts"));
}

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
fn tests_impact_json_accepts_structured_symbol_entrypoint() {
    let root = fixture_root("tests-impact-symbol");
    let options = json!({
        "root": root,
        "includeSymbols": true,
        "entrypoints": [{ "file": "utils.mts", "symbol": "parseDate" }]
    })
    .to_string();
    let output = tests_impact_json_impl(options).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();
    let test_files: Vec<&str> = selected
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(
        test_files,
        vec![
            "barrel-consumer.test.mts",
            "excluded-private-caller.test.mts",
            "helper-export.test.mts",
            "other.test.mts",
            "private-barrel-caller-with-export.test.mts",
            "private-caller-with-export.test.mts"
        ]
    );
    assert!(selected
        .iter()
        .all(|test| test["reasons"][0]["changed_file"] == "utils.mts#parseDate"));
}

#[test]
fn dependents_json_accepts_structured_symbol_file() {
    let root = fixture_root("tests-impact-symbol");
    let options = json!({
        "root": root,
        "includeSymbols": true,
        "files": [{ "file": "utils.mts", "symbol": "parseDate" }]
    })
    .to_string();
    let output = dependents_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    let files = value["files"].as_array().unwrap();
    assert!(files
        .iter()
        .any(|file| file["file"] == "other.mts" && file["symbol"] == "parse"));
    assert!(!files
        .iter()
        .any(|file| file["path"] == "service.test.mts"));
}

#[test]
fn tests_plan_json_with_diff_content() {
    let root = fixture_root("tests-impact-diff");
    let diff = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/tests-impact-diff/fixture/sample.diff"),
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
