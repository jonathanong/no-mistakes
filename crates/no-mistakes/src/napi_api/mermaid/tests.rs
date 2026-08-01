use super::super::validate_mermaid_markdown_json_impl;
use serde_json::{json, Value};
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/markdown-mermaid-validation")
        .join(name);
    std::fs::read_to_string(path).expect("Mermaid Markdown fixture should be readable")
}

#[test]
fn serializes_the_public_validation_result() {
    let options = json!({
        "content": fixture("invalid-flowchart.md"),
        "file": "docs/flow.md"
    });
    let output = validate_mermaid_markdown_json_impl(options.to_string()).unwrap();
    let output: Value = serde_json::from_str(&output).unwrap();

    assert_eq!(output["valid"], false);
    assert_eq!(output["diagramCount"], 1);
    assert_eq!(output["diagnostics"][0]["code"], "invalid-syntax");
    assert_eq!(output["diagnostics"][0]["file"], "docs/flow.md");
    assert_eq!(output["diagnostics"][0]["fenceLine"], 3);
}

#[test]
fn validates_mdx_jsx_children_without_a_blank_line() {
    let options = json!({
        "content": fixture("jsx-adjacent-invalid.mdx"),
        "file": "docs/component.mdx"
    });
    let output = validate_mermaid_markdown_json_impl(options.to_string()).unwrap();
    let output: Value = serde_json::from_str(&output).unwrap();

    assert_eq!(output["valid"], false);
    assert_eq!(output["diagramCount"], 1);
    assert_eq!(output["diagnostics"][0]["code"], "invalid-syntax");
    assert_eq!(output["diagnostics"][0]["fenceLine"], 4);
}

#[test]
fn rejects_missing_content_and_unknown_options() {
    let missing_content = validate_mermaid_markdown_json_impl(json!({}).to_string()).unwrap_err();
    assert!(missing_content
        .to_string()
        .contains("missing field `content`"));

    let unknown = validate_mermaid_markdown_json_impl(
        json!({ "content": fixture("valid.md"), "unknown": true }).to_string(),
    )
    .unwrap_err();
    assert!(unknown.to_string().contains("unknown field `unknown`"));
}
