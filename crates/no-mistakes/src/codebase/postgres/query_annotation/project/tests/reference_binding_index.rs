#[test]
fn saved_reference_binding_index_retains_findings_and_reduces_comparisons() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-shared-record-dag/reference-binding-index");
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected-report.json")).unwrap())
            .unwrap();
    let (off, disabled) = crate::benchmark_support::check_json_observed(&root, false).unwrap();
    let (on, enabled) = crate::benchmark_support::check_json_observed(&root, true).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&off).unwrap(),
        expected
    );
    assert_eq!(off, on);
    assert!(disabled.work.is_empty());
    // Legacy main performs 148 comparisons, including unrelated scalar/function bindings.
    let comparisons = enabled.work["postgres.annotation.prefix_walks"];
    assert!(comparisons > 0 && comparisons <= 24, "{comparisons}");
    assert_eq!(expected["rules"].as_array().unwrap().len(), 2);
    assert_eq!(expected["rules"][0]["line"], 42);
    assert_eq!(expected["rules"][1]["line"], 43);
}
