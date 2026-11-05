use super::*;

fn partial_fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check")
            .join(name),
    )
}

#[test]
fn analyze_project_check_matches_standalone_partial_report() {
    for name in ["partial-source", "partial-playwright"] {
        let root = partial_fixture(name);
        let result = analyze_project_check_result(&root);
        assert_eq!(result, standalone_check_result(&root));
        assert!(!result["incomplete"].as_array().unwrap().is_empty());
    }
}

#[test]
fn independent_check_scopes_share_sources_without_sharing_failures() {
    let root = partial_fixture("partial-source");
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                {"type": "check", "config": root.join(".no-mistakes.yml")},
                {"type": "check", "config": root.join("valid.yml")}
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let output: Value = serde_json::from_str(&output).unwrap();
    assert!(output["reports"][0]["result"]["incomplete"].is_array());
    assert!(
        output["reports"][1]["result"].get("incomplete").is_none(),
        "{output}"
    );
}

#[path = "graph_partial.rs"]
mod graph_partial;

#[test]
fn ordinary_queue_reports_do_not_inherit_aggregate_check_tolerance() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/queue-ast-hop/partial-failures/fixture"),
    );
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({"root": root, "reports": [{"type": "check"}]}).to_string(),
    ))
    .unwrap();
    let output: Value = serde_json::from_str(&output).unwrap();
    assert!(output["reports"][0]["result"]["incomplete"].is_array());
    let error = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({"root": root, "reports": [{"type": "check"}, {"type": "queueCheck"}]}).to_string(),
    ))
    .unwrap_err();
    assert!(error.to_string().contains("broken.ts"), "{error}");
}
