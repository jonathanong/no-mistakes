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
