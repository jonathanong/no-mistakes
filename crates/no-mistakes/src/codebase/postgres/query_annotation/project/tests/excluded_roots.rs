use super::demand_followups::{fixture, observed_check, observed_check_with_config};

#[test]
fn scoped_executor_keeps_literal_context_from_excluded_callers() {
    let root = fixture("helper-tracing-reachability");
    let (report, diagnostics) = observed_check(&root);
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 2, "{report}");
    assert!(
        findings.iter().any(|finding| {
            finding["rule"] == "postgres-require-query-annotation"
                && finding["file"] == "src/helper.mts"
                && finding["line"] == 3
        }),
        "{report}"
    );
    assert_eq!(diagnostics.work["postgres.annotation.roots"], 5);
}

#[test]
fn absent_scoped_seeds_skip_module_evaluation_and_target_resolution() {
    let root = fixture("helper-tracing-excluded-roots");
    let (report, diagnostics) =
        observed_check_with_config(&root, Some(root.join("include-none.yml")));
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    assert_eq!(
        diagnostics
            .work
            .get("postgres.annotation.roots")
            .copied()
            .unwrap_or(0),
        0
    );
    assert!(!diagnostics.work.contains_key("postgres.annotation.module"));
    assert!(!diagnostics
        .work
        .contains_key("postgres.annotation.target_resolutions"));
}

#[test]
fn scope_excluded_unimported_test_is_never_an_annotation_seed() {
    let root = fixture("helper-tracing-excluded-roots");
    let (report, diagnostics) = observed_check(&root);
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    assert_eq!(report["rules"].as_array().unwrap().len(), 1, "{report}");
    assert_eq!(diagnostics.work["postgres.annotation.roots"], 1);
    assert_eq!(diagnostics.work["postgres.annotation.module"], 1);
}

#[test]
fn include_misses_and_other_rule_scopes_do_not_widen_annotation_seeds() {
    let root = fixture("helper-tracing-excluded-roots");
    let (expected, _) = observed_check(&root);
    for config in ["include-only.yml", "mixed-rules.yml"] {
        let (report, diagnostics) = observed_check_with_config(&root, Some(root.join(config)));
        assert_eq!(report, expected, "{config}");
        assert_eq!(diagnostics.work["postgres.annotation.roots"], 1, "{config}");
        assert_eq!(
            diagnostics.work["postgres.annotation.module"], 1,
            "{config}"
        );
    }
}

#[test]
fn configured_excluded_dependency_preserves_each_root_context() {
    let root = fixture("helper-tracing-shared-initials");
    let (report, diagnostics) = observed_check(&root);
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    assert_eq!(diagnostics.work["postgres.annotation.roots"], 2);
    assert_eq!(diagnostics.work["postgres.annotation.module"], 4);
}

#[test]
fn excluded_scalar_preparation_preserves_many_root_contexts() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-excluded-shared-initials");
    let (report, diagnostics) = observed_check(&root);
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    assert_eq!(diagnostics.work["postgres.annotation.roots"], 32);
    assert_eq!(diagnostics.work["postgres.annotation.module"], 64);
}
