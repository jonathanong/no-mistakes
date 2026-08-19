use super::*;
use serde_json::{json, Value};

fn fixture_root(name: &str) -> String {
    crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
    .display()
    .to_string()
}

#[test]
fn analyze_project_importers_report_lists_direct_importers() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("simple"),
            "reports": [{ "type": "importers", "id": "who", "file": "b.mts" }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["reports"][0]["id"], "who");
    assert_eq!(value["reports"][0]["type"], "importers");
    let importers = value["reports"][0]["result"]["directImporters"]
        .as_array()
        .unwrap();
    assert!(importers.iter().any(|importer| importer == "a.mts"));
}

#[cfg(feature = "mermaid-validation")]
#[test]
fn analyze_project_validates_mermaid_markdown_in_memory() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("simple"),
            "reports": [{
                "type": "validateMermaidMarkdown",
                "content": "```mermaid\nflowchart LR\n  A --> B\n```"
            }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["reports"][0]["type"], "validateMermaidMarkdown");
    assert_eq!(value["reports"][0]["result"]["valid"], true);
    assert_eq!(value["reports"][0]["result"]["diagramCount"], 1);
}

#[test]
fn analyze_project_tests_comment_renders_inline_plan() {
    let plan = json!({
        "selected_tests": [{
            "test_file": "tests/app.test.ts",
            "confidence": "high",
            "reasons": [{
                "changed_file": "src/app.ts",
                "path": ["src/app.ts", "tests/app.test.ts"],
                "via": ["Test"]
            }]
        }],
        "warnings": [],
        "fallback_triggered": false,
        "fallback_reason": null
    });
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("simple"),
            "reports": [{ "type": "testsComment", "planJson": plan }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["reports"][0]["type"], "testsComment");
    assert!(value["reports"][0]["result"]
        .as_str()
        .unwrap()
        .contains("tests/app.test.ts"));
}

#[test]
fn analyze_project_additive_importers_report_keeps_dependents_fields() {
    let root = fixture_root("simple");
    let baseline = analyze_project_json_impl(
        json!({
            "root": root,
            "reports": [{ "type": "dependents", "id": "deps", "files": ["b.mts"] }]
        })
        .to_string(),
    )
    .unwrap();
    let mixed = analyze_project_json_impl(
        json!({
            "root": root,
            "reports": [
                { "type": "dependents", "id": "deps", "files": ["b.mts"] },
                { "type": "importers", "id": "who", "file": "b.mts" }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let baseline: Value = serde_json::from_str(&baseline).unwrap();
    let mixed: Value = serde_json::from_str(&mixed).unwrap();
    assert_eq!(
        mixed["reports"][0]["result"],
        baseline["reports"][0]["result"]
    );
    let standalone = crate::napi_api::queries::importers_json_impl(
        json!({ "root": root, "file": "b.mts" }).to_string(),
    )
    .unwrap();
    let standalone: Value = serde_json::from_str(&standalone).unwrap();
    assert_eq!(mixed["reports"][1]["result"], standalone);
}
