use std::path::PathBuf;

#[test]
fn saved_shared_record_fixture_retains_report_and_exposes_walk_work() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-shared-record-dag/shared-object-dag-12");
    let (off, disabled) = crate::benchmark_support::check_json_observed(&root, false).unwrap();
    let (on, enabled) = crate::benchmark_support::check_json_observed(&root, true).unwrap();
    assert_eq!(off, on);
    assert!(disabled.work.is_empty());
    assert_eq!(enabled.work["postgres.annotation.root"], 1);
    assert_eq!(enabled.work["postgres.annotation.roots_completed"], 1);
    assert_eq!(
        enabled.work["postgres.annotation.module_initializations_completed"],
        1
    );
    assert!(enabled.work["postgres.annotation.prefix_nodes"] > 1);
    assert!(enabled.work["postgres.annotation.prefix_nodes"] <= 2_000);
    assert!(enabled.work["postgres.annotation.prefix_containers"] > 1);
    assert_eq!(
        enabled.work["postgres.annotation.prefix_walks"],
        enabled.work["postgres.annotation.prefix_walks_completed"],
    );
}

#[test]
fn reference_free_entry_scopes_skip_prefix_walks_without_changing_the_saved_report() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-shared-record-dag/reference-free-scopes");
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected-report.json")).unwrap())
            .unwrap();
    let (off, disabled) = crate::benchmark_support::check_json_observed(&root, false).unwrap();
    let (on, enabled) = crate::benchmark_support::check_json_observed(&root, true).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&off).unwrap(),
        expected
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&on).unwrap(),
        expected
    );
    assert!(disabled.work.is_empty());
    assert_eq!(
        enabled
            .work
            .get("postgres.annotation.prefix_walks")
            .copied()
            .unwrap_or_default(),
        0,
    );
    assert_eq!(enabled.work["postgres.annotation.root"], 1);
}
