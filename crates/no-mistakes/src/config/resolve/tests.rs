use super::resolve_config;
use std::path::PathBuf;

#[test]
fn resolve_config_reports_named_triggers_and_coverage_gates() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/config/named-full-suite-triggers");
    let report = resolve_config(&root, None).unwrap();
    assert!(report
        .vitest_full_suite_triggers
        .iter()
        .any(|trigger| trigger.name == "postgres-resources" && trigger.source == "triggers"));
    assert!(report.playwright.coverage_routes);
    assert!(report.playwright.coverage_selectors);
}
