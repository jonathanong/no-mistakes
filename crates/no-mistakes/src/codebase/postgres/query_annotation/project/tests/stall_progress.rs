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
    assert!(!enabled
        .work
        .contains_key("postgres.annotation.prefix_compare_arm"));
    assert!(!enabled
        .work
        .contains_key("postgres.annotation.prefix_observer_batches"));
}

#[test]
fn legacy_bindings_share_arm_publications_without_changing_findings() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-shared-record-dag/observer-batching");
    let plain = crate::benchmark_support::check_json(&root).unwrap();
    let (off, disabled) = crate::benchmark_support::check_json_observed(&root, false).unwrap();
    let (on, enabled) = crate::benchmark_support::check_json_observed(&root, true).unwrap();
    assert_eq!(plain, off);
    assert_eq!(plain, on);
    assert!(disabled.work.is_empty());
    let report: serde_json::Value = serde_json::from_str(&on).unwrap();
    assert_eq!(report["rules"].as_array().unwrap().len(), 1);
    assert_eq!(
        report["rules"][0]["rule"],
        "postgres-require-query-annotation"
    );
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/performance/annotation-shared-record-dag/observer-batching/root.mts"
    ));
    let unsafe_line = source
        .lines()
        .position(|line| line.ends_with("// unsafe-query"))
        .unwrap()
        + 1;
    assert_eq!(report["rules"][0]["line"], unsafe_line);
    let work = enabled.work;
    // Reference-bearing aliases keep this legacy workload valid after binding pruning.
    assert!(work["postgres.annotation.prefix_compare"] > 64);
    assert_eq!(
        work["postgres.annotation.prefix_walks"],
        work["postgres.annotation.prefix_compare"]
    );
    assert_eq!(
        work["postgres.annotation.prefix_walks_completed"],
        work["postgres.annotation.prefix_walks"]
    );
    assert_eq!(work["postgres.annotation.prefix_compare_arm"], 2);
    assert_eq!(work["postgres.annotation.prefix_observer_batches"], 2);
}

#[test]
fn saved_fixture_has_opt_in_phase_timings_and_identical_findings() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-timings");
    let (off, disabled) = crate::benchmark_support::check_json_observed(&root, false).unwrap();
    let (on, enabled) = crate::benchmark_support::check_json_observed(&root, true).unwrap();
    assert_eq!(off, on);
    let report: serde_json::Value = serde_json::from_str(&on).unwrap();
    assert_eq!(report["rules"].as_array().unwrap().len(), 1);
    assert_eq!(
        report["rules"][0]["rule"],
        "postgres-require-query-annotation"
    );
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/performance/annotation-timings/root.mts"
    ));
    let unsafe_line = source
        .lines()
        .position(|line| line.ends_with("// unsafe-query"))
        .unwrap()
        + 1;
    assert_eq!(report["rules"][0]["line"], unsafe_line);
    for label in [
        "postgres.annotation.root",
        "postgres.annotation.module",
        "postgres.annotation.snapshot",
        "postgres.annotation.snapshot_clone",
        "postgres.annotation.snapshot_restore",
        "postgres.annotation.entrypoint",
        "postgres.annotation.alternatives_snapshot",
        "postgres.annotation.alternatives_snapshot_restore",
        "postgres.annotation.alternative_arm",
        "postgres.annotation.alternatives_join",
        "postgres.annotation.alternatives_finish",
        "postgres.annotation.reachability_walk",
    ] {
        assert!(
            !disabled.timings.iter().any(|timing| timing.label == label),
            "{label}"
        );
        let timing = enabled
            .timings
            .iter()
            .find(|timing| timing.label == label)
            .unwrap_or_else(|| panic!("missing timing: {label}"));
        assert_eq!(
            timing.kind,
            crate::diagnostics::TimingKind::Parallel,
            "{label}"
        );
    }
    assert!(enabled
        .timings
        .iter()
        .all(|timing| !timing.label.contains("root.mts")));
}
