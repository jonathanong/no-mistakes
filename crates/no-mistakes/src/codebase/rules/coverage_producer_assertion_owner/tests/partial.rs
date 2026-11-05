use super::*;

#[test]
fn failed_required_sources_stay_incomplete_under_ignore_with_healthy_applications_retained() {
    let saved = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/coverage-owner-partial");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let report = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({"root": root}).to_string(),
    ))
    .unwrap();
    let report: serde_json::Value = serde_json::from_str(&report).unwrap();
    let failures = report["incomplete"].as_array().unwrap();
    assert_eq!(failures.len(), 1, "{report}");
    assert_eq!(failures[0]["consumer"], RULE_ID);
    assert!(failures[0]["file"].as_str().unwrap().ends_with("broken.ts"));
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{report}");
    assert_eq!(findings[0]["target"], "healthy");
    assert!(crate::codebase::rules::run_check(root, None, None).is_err());
}
