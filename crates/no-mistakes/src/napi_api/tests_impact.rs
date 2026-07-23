include!("tests_impact/resources.rs");
include!("tests_impact/vitest_commonjs_projects.rs");
include!("tests_impact/vitest_setup.rs");

#[test]
fn tests_impact_json_preserves_configured_native_test_projects() {
    for (fixture, test) in [
        (
            "dotnet-test-plan",
            "dotnet-clients/tests/App.Tests/FeedServiceTests.cs",
        ),
        (
            "swift-test-plan",
            "swift-clients/core/Tests/VouchaCoreTests/APIClientTests.swift",
        ),
    ] {
        let output = tests_impact_json_impl(
            json!({
                "root": fixture_root(fixture),
                "entrypoints": [test]
            })
            .to_string(),
        )
        .unwrap();
        let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(plan["selected_tests"][0]["test_file"], test, "{plan:#}");
        assert_eq!(plan["selected_tests"][0]["reasons"][0]["via"][0], "self");
    }
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
fn tests_impact_json_forces_explicit_ignored_tsconfig_for_vitest_setups_once() {
    let fixture = crate::test_support::materialize_gitignore_fixture(
        "tests-impact-forced-tsconfig",
    );
    let root = fixture.path().canonicalize().unwrap();
    crate::ast::begin_parse_count(&root);
    let output = crate::ast::with_request_parse_cache(|| {
        tests_impact_json_impl(
            json!({
                "root": root,
                "tsconfig": "tsconfig.custom.json",
                "entrypoints": ["setup/helper.ts"]
            })
            .to_string(),
        )
    })
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(plan["selected_tests"].as_array().unwrap().iter().any(|test| {
        test["test_file"] == "tests/custom.test.ts"
    }), "{plan:#?}");
    for path in [
        root.join("vitest.config.ts"),
        root.join("setup/custom.ts"),
        root.join("setup/helper.ts"),
    ] {
        assert_eq!(counts.get(&path), Some(&1), "{counts:#?}");
    }
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
    assert!(!files.iter().any(|file| file["path"] == "service.test.mts"));
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
