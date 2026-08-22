use super::resolve_config;
use serde_json::json;
use std::path::PathBuf;

fn named_triggers_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/config/named-full-suite-triggers")
}

#[test]
fn resolve_config_reports_named_triggers_and_coverage_gates() {
    let report = resolve_config(&named_triggers_fixture(), None).unwrap();
    assert!(report
        .vitest_full_suite_triggers
        .iter()
        .any(|trigger| trigger.name == "postgres-resources" && trigger.source == "triggers"));
    assert!(report.playwright.coverage_routes);
    assert!(report.playwright.coverage_selectors);
}

#[test]
fn resolve_config_json_impl_returns_the_same_named_triggers() {
    let root = named_triggers_fixture();
    let output =
        crate::napi_api::resolve_config_json_impl(json!({ "root": root }).to_string()).unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    let names: Vec<&str> = report["vitestFullSuiteTriggers"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|trigger| trigger["name"].as_str())
        .collect();
    assert!(names.contains(&"postgres-resources"));
}
