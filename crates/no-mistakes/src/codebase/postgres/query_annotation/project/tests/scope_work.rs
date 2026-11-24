use super::demand_followups::{fixture, observed_check};

#[test]
fn isolated_scope_pruning_reports_legacy_and_selected_root_counts() {
    let (_, diagnostics) = observed_check(&fixture("helper-tracing-excluded-roots"));
    let work = diagnostics.work;
    assert_eq!(work["postgres.annotation.roots_before_scoping"], 2);
    assert_eq!(work["postgres.annotation.roots_after_scoping"], 1);
    assert_eq!(work["postgres.annotation.excluded_roots_retained"], 0);
    assert_eq!(work["postgres.annotation.excluded_roots_pruned"], 1);
}

#[test]
fn excluded_root_counts_do_not_include_in_scope_importers_that_are_also_pruned() {
    let (_, diagnostics) = observed_check(&fixture("helper-tracing-scope-root-counters"));
    let work = diagnostics.work;
    assert_eq!(work["postgres.annotation.roots_before_scoping"], 3);
    assert_eq!(work["postgres.annotation.roots_after_scoping"], 1);
    assert_eq!(work["postgres.annotation.excluded_roots_retained"], 0);
    assert_eq!(work["postgres.annotation.excluded_roots_pruned"], 1);
}

#[test]
fn excluded_callers_of_large_helpers_remain_roots_without_shared_cache_work() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-excluded-callers-large");
    let (report, diagnostics) = observed_check(&root);
    assert_eq!(report["rules"].as_array().unwrap().len(), 1, "{report}");
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    let work = diagnostics.work;
    assert_eq!(work["postgres.annotation.roots_before_scoping"], 65);
    assert_eq!(work["postgres.annotation.roots_after_scoping"], 65);
    assert_eq!(work["postgres.annotation.excluded_roots_retained"], 64);
    assert_eq!(work["postgres.annotation.excluded_roots_pruned"], 0);
    assert_eq!(work["postgres.annotation.shared_init.disabled"], 1);
    assert_eq!(
        work.get("postgres.annotation.prefix_walks")
            .copied()
            .unwrap_or(0),
        0
    );
    let builds = work["postgres.annotation.dense_prefix_summary_builds"];
    let skips = work["postgres.annotation.prefix_unchanged_binding_skips"];
    assert!(builds > 0 && builds <= skips, "{work:?}");
    for metric in ["cache_hits", "cache_misses", "builds"] {
        assert_eq!(
            work[&*format!("postgres.annotation.shared_init.{metric}")],
            0
        );
    }
    assert_eq!(
        work["postgres.annotation.module_initializations"],
        work["postgres.annotation.module"]
    );
    assert!(
        work["postgres.annotation.module_initializations"] <= 300,
        "multi-entrypoint roots must reuse pure imported module initializations instead of repeating them per entrypoint: {work:?}"
    );
    assert!(diagnostics.timings.iter().any(|timing| {
        timing.label == "postgres.annotation.shared_init.build" && timing.duration.is_zero()
    }));
}

#[test]
fn multi_entrypoint_root_reuses_pure_imports_and_keeps_impure_isolated() {
    let (report, diagnostics) = observed_check(&fixture("helper-tracing-entrypoint-reuse"));
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    let mut actual: Vec<(&str, &str, u64)> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            (
                finding["rule"].as_str().unwrap(),
                finding["file"].as_str().unwrap(),
                finding["line"].as_u64().unwrap(),
            )
        })
        .collect();
    actual.sort_unstable();
    assert_eq!(
        actual,
        vec![
            ("postgres-require-query-annotation", "src/root.mts", 42),
            (
                "postgres-require-query-annotation",
                "src/shared/impure.mts",
                3
            ),
        ],
        "{report}"
    );
    let work = diagnostics.work;
    assert_eq!(work["postgres.annotation.roots_after_scoping"], 2);
    assert!(
        work["postgres.annotation.module_initializations"] <= 8,
        "pure direct and lazy transitive imports must not re-initialize on every entrypoint: {work:?}"
    );
}

#[test]
fn unrelated_large_excluded_test_modules_are_pruned_before_evaluation() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-excluded-callers-unrelated");
    let (_, diagnostics) = observed_check(&root);
    let work = diagnostics.work;
    assert_eq!(work["postgres.annotation.roots_before_scoping"], 65);
    assert_eq!(work["postgres.annotation.roots_after_scoping"], 1);
    assert_eq!(work["postgres.annotation.excluded_roots_retained"], 0);
    assert_eq!(work["postgres.annotation.excluded_roots_pruned"], 64);
}
