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
