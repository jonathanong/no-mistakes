use crate::diagnostics::DiagnosticsSnapshot;

fn observed(name: &str) -> (serde_json::Value, DiagnosticsSnapshot) {
    let root =
        crate::test_support::rule_fixture_root("postgres-require-query-annotation").join(name);
    let (output, diagnostics) = crate::benchmark_support::check_json_observed(&root, true).unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["warnings"], serde_json::json!([]), "{output}");
    (report, diagnostics)
}

fn work(diagnostics: &DiagnosticsSnapshot, metric: &str) -> u64 {
    diagnostics.work.get(metric).copied().unwrap_or_default()
}

fn one_finding(report: &serde_json::Value, file: &str, line: u64) {
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{report}");
    assert_eq!(findings[0]["rule"], "postgres-require-query-annotation");
    assert_eq!(findings[0]["file"], file);
    assert_eq!(findings[0]["line"], line);
    assert!(findings[0]["message"]
        .as_str()
        .unwrap()
        .ends_with("query execution must start with a /* name */ annotation"));
}

#[test]
fn selected_workspace_helper_keeps_contextual_annotation_finding() {
    let (report, diagnostics) = observed("workspace-fallback-selected-helper");
    one_finding(&report, "packages/executor/src/index.mts", 2);
    assert!(work(&diagnostics, "postgres.annotation.workspace_fallback_edges") > 0);
}

#[test]
fn project_scope_without_option_include_blocks_unrelated_excluded_workspace_helper() {
    let (report, diagnostics) = observed("workspace-fallback-excluded-test");
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    assert_eq!(
        work(&diagnostics, "postgres.annotation.workspace_fallback_edges"),
        0
    );
    // A missing target would also yield zero accepted edges. Require proof
    // that an actual workspace target was found and rejected by scope.
    assert!(
        work(
            &diagnostics,
            "postgres.annotation.workspace_fallback_blocked"
        ) > 0
    );
}

#[test]
fn excluded_importer_can_reach_configured_executor_package_subpath() {
    let (report, diagnostics) = observed("workspace-fallback-configured-target");
    one_finding(&report, "packages/db/src/read.mts", 2);
    assert!(work(&diagnostics, "postgres.annotation.workspace_fallback_edges") > 0);
}
