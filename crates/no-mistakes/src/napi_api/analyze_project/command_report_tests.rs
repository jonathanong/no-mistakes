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
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": fixture_root("simple"),
            "reports": [{ "type": "importers", "id": "who", "file": "b.mts" }]
        })
        .to_string(),
    ))
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
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": fixture_root("simple"),
            "reports": [{
                "type": "validateMermaidMarkdown",
                "content": "```mermaid\nflowchart LR\n  A --> B\n```"
            }]
        })
        .to_string(),
    ))
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
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": fixture_root("simple"),
            "reports": [{ "type": "testsComment", "planJson": plan }]
        })
        .to_string(),
    ))
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
    let baseline = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [{ "type": "dependents", "id": "deps", "files": ["b.mts"] }]
        })
        .to_string(),
    ))
    .unwrap();
    let mixed = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                { "type": "dependents", "id": "deps", "files": ["b.mts"] },
                { "type": "importers", "id": "who", "file": "b.mts" }
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let baseline: Value = serde_json::from_str(&baseline).unwrap();
    let mixed: Value = serde_json::from_str(&mixed).unwrap();
    assert_eq!(
        mixed["reports"][0]["result"],
        baseline["reports"][0]["result"]
    );
    let standalone =
        crate::napi_api::queries::importers_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root, "file": "b.mts" }).to_string(),
        ))
        .unwrap();
    let standalone: Value = serde_json::from_str(&standalone).unwrap();
    assert_eq!(mixed["reports"][1]["result"], standalone);
}

#[test]
fn command_options_forward_top_level_tsconfig_and_config() {
    let root = fixture_root("simple");
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": root,
            "tsconfig": "tsconfig.json",
            "config": "no-mistakes.json",
            "reports": [
                { "type": "importers", "file": "b.mts" },
                { "type": "ciImpact", "file": "b.mts" },
                { "type": "testsPlan" },
                { "type": "lockfileDiff" },
                { "type": "registryExtension" },
                { "type": "importers", "file": "b.mts", "tsconfig": "keep.json" },
                { "type": "ciImpact", "file": "b.mts", "config": "keep.yml" }
            ]
        })
        .to_string(),
    )
    .unwrap();

    let importers = options::command_options(&options.reports[0], &options).unwrap();
    assert_eq!(importers["root"], root);
    assert_eq!(importers["tsconfig"], format!("{root}/tsconfig.json"));
    assert!(importers.get("config").is_none());

    let ci = options::command_options(&options.reports[1], &options).unwrap();
    assert_eq!(ci["config"], format!("{root}/no-mistakes.json"));
    assert!(ci.get("tsconfig").is_none());

    let tests_plan = options::command_options(&options.reports[2], &options).unwrap();
    assert_eq!(tests_plan["tsconfig"], format!("{root}/tsconfig.json"));
    assert_eq!(tests_plan["config"], format!("{root}/no-mistakes.json"));

    let lockfile = options::command_options(&options.reports[3], &options).unwrap();
    assert!(lockfile.get("tsconfig").is_none());
    assert!(lockfile.get("config").is_none());

    let registry = options::command_options(&options.reports[4], &options).unwrap();
    assert!(registry.get("tsconfig").is_none());
    assert_eq!(registry["config"], format!("{root}/no-mistakes.json"));

    let keep_tsconfig = options::command_options(&options.reports[5], &options).unwrap();
    assert_eq!(keep_tsconfig["tsconfig"], "keep.json");

    let keep_config = options::command_options(&options.reports[6], &options).unwrap();
    assert_eq!(keep_config["config"], "keep.yml");
}
