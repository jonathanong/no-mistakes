use super::demand_followups::{fixture, observed_check, observed_check_with_config};

#[test]
fn scope_excluded_unimported_test_is_never_an_annotation_root() {
    let root = fixture("helper-tracing-excluded-roots");
    let (report, diagnostics) = observed_check(&root);
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    assert_eq!(report["rules"].as_array().unwrap().len(), 1, "{report}");
    assert_eq!(diagnostics.work["postgres.annotation.roots"], 1);
    assert_eq!(diagnostics.work["postgres.annotation.module"], 1);
}

#[test]
fn include_misses_and_other_rule_scopes_do_not_widen_annotation_roots() {
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
fn configured_excluded_dependency_initializes_once_across_roots() {
    let root = fixture("helper-tracing-shared-initials");
    let (report, diagnostics) = observed_check(&root);
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    assert_eq!(diagnostics.work["postgres.annotation.roots"], 2);
    assert_eq!(diagnostics.work["postgres.annotation.module"], 3);
    assert_eq!(
        diagnostics.work["postgres.annotation.shared_module_initializations"],
        1
    );
}

#[test]
fn expensive_excluded_scalar_preparation_is_shared_across_many_roots() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-excluded-shared-initials");
    let (report, diagnostics) = observed_check(&root);
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    assert_eq!(diagnostics.work["postgres.annotation.roots"], 32);
    assert_eq!(diagnostics.work["postgres.annotation.module"], 33);
    assert_eq!(
        diagnostics.work["postgres.annotation.shared_module_initializations"],
        1
    );
}
