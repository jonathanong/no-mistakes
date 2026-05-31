use serde_json::json;

use super::*;

#[test]
fn dependencies_json_preserves_hash_in_structured_entrypoint_file() {
    let options = json!({
        "root": fixture_root("symbol-export"),
        "includeSymbols": true,
        "files": [{ "file": "hash#entry.mts", "symbol": "run" }]
    })
    .to_string();

    let output = dependencies_json_impl(options).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(value["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| file["file"] == "source.mts" && file["symbol"] == "alpha"));
}

#[test]
fn tests_impact_json_preserves_hash_in_structured_entrypoint_file() {
    let options = json!({
        "root": fixture_root("tests-impact-symbol"),
        "includeSymbols": true,
        "entrypoints": [{ "file": "hash#utils.mts", "symbol": "parseHashDate" }]
    })
    .to_string();

    let output = tests_impact_json_impl(options).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0]["test_file"], "hash-consumer.test.mts");
    assert_eq!(
        selected[0]["reasons"][0]["changed_file"],
        "hash#utils.mts#parseHashDate"
    );
}

#[test]
fn tests_plan_json_preserves_hash_in_structured_entrypoint_file() {
    let options = json!({
        "root": fixture_root("tests-impact-symbol"),
        "includeSymbols": true,
        "entrypoints": [{ "file": "hash#utils.mts", "symbol": "parseHashDate" }]
    })
    .to_string();

    let output = tests_plan_json_impl(options).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0]["test_file"], "hash-consumer.test.mts");
    assert_eq!(
        selected[0]["reasons"][0]["changed_file"],
        "hash#utils.mts#parseHashDate"
    );
}
