use crate::diagnostics::DiagnosticsSnapshot;

fn observed(name: &str, line: u64) -> DiagnosticsSnapshot {
    let root =
        crate::test_support::rule_fixture_root("postgres-require-query-annotation").join(name);
    let (output, diagnostics) = crate::benchmark_support::check_json_observed(&root, true).unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["warnings"], serde_json::json!([]), "{output}");
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{output}");
    assert_eq!(findings[0]["rule"], "postgres-require-query-annotation");
    assert_eq!(findings[0]["file"], "src/a.mts");
    assert_eq!(findings[0]["line"], line);
    assert!(findings[0]["message"]
        .as_str()
        .unwrap()
        .ends_with("query execution must start with a /* name */ annotation"));
    assert!(
        diagnostics
            .work
            .get("postgres.annotation.workspace_fallback_edges")
            .copied()
            .unwrap_or_default()
            > 0
    );
    diagnostics
}

#[test]
fn selected_annotation_keeps_sql_through_two_out_of_scope_workspace_helpers() {
    // unanalyzableSql:ignore makes erased SQL a silent false negative. A real
    // selected finding is required; an empty report cannot satisfy this guard.
    observed("workspace-fallback-transitive-chain", 3);
}

#[test]
fn transitive_workspace_reexports_handle_diamond_and_cycle_without_type_only_widening() {
    let diagnostics = observed("workspace-fallback-transitive-graph", 4);
    // The prepared type-only package has a known runtime edge to @p/noise.
    // It must remain blocked because the selected source imports only its type.
    assert!(
        diagnostics
            .work
            .get("postgres.annotation.workspace_fallback_blocked")
            .copied()
            .unwrap_or_default()
            > 0
    );
}
