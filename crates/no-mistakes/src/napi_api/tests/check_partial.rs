use super::*;

#[test]
fn async_check_report_retains_findings_with_incomplete_consumers() {
    for fixture in ["partial-source", "partial-playwright"] {
        let report: serde_json::Value = serde_json::from_str(
            &check_json_impl(json!({"root": static_check_fixture(fixture)})).unwrap(),
        )
        .unwrap();
        assert!(!report["incomplete"].as_array().unwrap().is_empty());
        assert!(report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| { finding["rule"] == "no-empty-or-comments-only-files" }));
    }
}

#[test]
fn check_report_omits_incomplete_for_successful_prerequisites() {
    let root = static_check_fixture("partial-source");
    let report: serde_json::Value = serde_json::from_str(
        &check_json_impl(json!({"root": root, "config": root.join("valid.yml")})).unwrap(),
    )
    .unwrap();
    assert!(report.get("incomplete").is_none(), "{report}");
}

#[test]
fn foreign_receiver_budget_failure_preserves_file_and_programmatic_parity() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/foreign-receiver-work-budget/fixture");
    let report: serde_json::Value = serde_json::from_str(
        &check_json_impl(json!({"root": root, "config": root.join("independent-config.yml")}))
            .unwrap(),
    )
    .unwrap();
    let analyzed =
        crate::napi_api::analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root,
                "config": root.join("independent-config.yml"),
                "reports": [{"type": "check"}]
            })
            .to_string(),
        ))
        .unwrap();
    let analyzed: serde_json::Value = serde_json::from_str(&analyzed).unwrap();
    assert_eq!(report, analyzed["reports"][0]["result"]);
    let failures = report["incomplete"].as_array().unwrap();
    assert!(
        failures.iter().any(|failure| {
            failure["consumer"] == "unconstructed-error-class"
                && failure["file"].as_str().is_some_and(|file| {
                    std::path::Path::new(file)
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .starts_with("shared-")
                })
                && failure["message"].as_str().is_some_and(|message| {
                    message.contains("per-file work budget")
                        && message.contains("narrower analysis root")
                })
        }),
        "{report}"
    );
    assert!(
        !report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| { finding["rule"] == "unconstructed-error-class" }),
        "incomplete value flow cannot prove a finding: {report}"
    );
    assert!(
        report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| { finding["rule"] == "forbidden-lint-suppressions" }),
        "complete same-file source facts must survive: {report}"
    );
    assert!(!failures
        .iter()
        .any(|failure| failure["consumer"] == "forbidden-lint-suppressions"));
}
