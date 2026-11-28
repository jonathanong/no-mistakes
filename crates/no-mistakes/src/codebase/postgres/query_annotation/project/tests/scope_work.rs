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
fn excluded_callers_of_large_helpers_share_pure_imported_modules() {
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
    assert_eq!(work["postgres.annotation.shared_init.disabled"], 0);
    assert!(
        work["postgres.annotation.shared_init.builds"] >= 1,
        "{work:?}"
    );
    assert!(
        work["postgres.annotation.shared_init.cache_hits"] >= 64,
        "{work:?}"
    );
    assert_eq!(
        work.get("postgres.annotation.prefix_walks")
            .copied()
            .unwrap_or(0),
        0
    );
    let builds = work["postgres.annotation.dense_prefix_summary_builds"];
    let skips = work["postgres.annotation.prefix_unchanged_binding_skips"];
    assert!(builds > 0 && builds <= skips, "{work:?}");
    assert_eq!(
        work["postgres.annotation.module_initializations"],
        work["postgres.annotation.module"]
    );
    assert!(
        work["postgres.annotation.module_initializations"] <= 70,
        "shared pure helper must initialize once across all 65 roots: {work:?}"
    );
}

#[test]
fn shared_roots_default_config_reuses_pure_imports_and_keeps_impure_isolated() {
    let (report, diagnostics) = observed_check(&fixture("helper-tracing-shared-roots-default"));
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
            (
                "postgres-require-query-annotation",
                "src/roots/root-15.mts",
                8
            ),
            (
                "postgres-require-query-annotation",
                "src/shared/impure.mts",
                3
            ),
        ],
        "{report}"
    );
    let work = diagnostics.work;
    assert_eq!(work["postgres.annotation.roots_after_scoping"], 17);
    assert_eq!(work["postgres.annotation.shared_init.disabled"], 0);
    assert_eq!(work["postgres.annotation.shared_init.builds"], 4);
    assert!(
        work["postgres.annotation.shared_init.cache_hits"] >= 56,
        "{work:?}"
    );
    assert!(
        work["postgres.annotation.module_initializations"] <= 24,
        "16 roots sharing 4 pure modules must initialize each pure module once: {work:?}"
    );
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
        work["postgres.annotation.module_initializations"] <= 12,
        "pure direct and lazy transitive imports must not re-initialize on every entrypoint: {work:?}"
    );
}

#[test]
fn disabled_multitask_root_with_impure_lazy_module_never_adds_initializations() {
    let (report, diagnostics) =
        observed_check(&fixture("helper-tracing-disabled-impure-multitask"));
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    let work = diagnostics.work;
    assert_eq!(work["postgres.annotation.roots_after_scoping"], 1);
    assert_eq!(work["postgres.annotation.shared_init.disabled"], 1);
    assert_eq!(
        work["postgres.annotation.module_initializations"], 5,
        "1 root + 4 task loads of impure module must not add a throwaway promotion init: {work:?}"
    );
}

#[test]
fn enabled_shared_init_with_impure_candidate_never_adds_initializations() {
    let (report, diagnostics) =
        observed_check(&fixture("helper-tracing-shared-init-impure-candidate"));
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    let work = diagnostics.work;
    assert_eq!(work["postgres.annotation.roots_after_scoping"], 2);
    assert_eq!(work["postgres.annotation.shared_init.disabled"], 0);
    assert_eq!(work["postgres.annotation.shared_init.builds"], 0);
    assert_eq!(work["postgres.annotation.shared_init.cache_misses"], 1);
    assert_eq!(
        work["postgres.annotation.module_initializations"], 4,
        "2 roots + 2 root loads of impure candidate must not add a throwaway build init: {work:?}"
    );
}

#[test]
fn enabled_shared_init_with_statically_pure_dynamically_impure_module_matches_main() {
    let (report, diagnostics) =
        observed_check(&fixture("helper-tracing-shared-init-dynamic-impure"));
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    let work = diagnostics.work;
    assert_eq!(work["postgres.annotation.roots_after_scoping"], 2);
    assert_eq!(work["postgres.annotation.shared_init.disabled"], 0);
    assert_eq!(work["postgres.annotation.shared_init.builds"], 0);
    assert_eq!(work["postgres.annotation.shared_init.cache_hits"], 0);
    assert_eq!(work["postgres.annotation.shared_init.cache_misses"], 1);
    assert_eq!(
        work["postgres.annotation.module_initializations"], 4,
        "statically pure but dynamically impure candidate must initialize once per root (matching main), never +1: {work:?}"
    );
}

#[test]
fn shared_init_disabled_reasons_report_disabled_counter() {
    let (_, no_shared) = observed_check(&fixture("helper-tracing-shared-init-no-shared-imports"));
    assert_eq!(
        no_shared.work["postgres.annotation.shared_init.disabled"],
        1
    );
    let (_, failed_sources) =
        observed_check(&fixture("helper-tracing-shared-init-failed-sql-sources"));
    assert_eq!(
        failed_sources.work["postgres.annotation.shared_init.disabled"],
        1
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

#[test]
fn lazy_transitive_shared_init_never_panics_and_matches_main() {
    let (report, diagnostics) =
        observed_check(&fixture("helper-tracing-shared-init-lazy-transitive"));
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    let work = diagnostics.work;
    assert_eq!(work["postgres.annotation.roots_after_scoping"], 3);
    assert_eq!(work["postgres.annotation.shared_init.disabled"], 0);
    assert_eq!(work["postgres.annotation.shared_init.builds"], 3);
    assert!(
        work["postgres.annotation.shared_init.cache_hits"] >= 3,
        "{work:?}"
    );
}

#[test]
fn local_promotion_preserves_untrusted_array_intrinsics_from_regexp_init() {
    let (report, diagnostics) = observed_check(&fixture("helper-tracing-entrypoint-reuse-regexp"));
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    let mut lines: Vec<u64> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["line"].as_u64().unwrap())
        .collect();
    lines.sort_unstable();
    assert_eq!(lines, vec![5, 9, 13, 17], "{report}");
    assert_eq!(
        diagnostics.work["postgres.annotation.module_initializations"],
        5
    );
}

#[test]
fn barrel_reexport_over_impure_leaf_preserves_effects_across_roots() {
    let (report, diagnostics) =
        observed_check(&fixture("helper-tracing-shared-init-reexport-impure"));
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    let mut actual: Vec<(&str, u64)> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            (
                finding["file"].as_str().unwrap(),
                finding["line"].as_u64().unwrap(),
            )
        })
        .collect();
    actual.sort_unstable();
    assert_eq!(actual, vec![("src/roots/root-a.mts", 8)], "{report}");
    let work = diagnostics.work;
    assert_eq!(work["postgres.annotation.shared_init.disabled"], 0);
    assert_eq!(work["postgres.annotation.shared_init.builds"], 0);
    assert_eq!(work["postgres.annotation.shared_init.cache_hits"], 0);
    assert_eq!(work["postgres.annotation.module_initializations"], 8);
}

#[test]
fn lazy_import_is_not_preinstalled_when_root_never_calls_lazy_export() {
    let (report, diagnostics) =
        observed_check(&fixture("helper-tracing-shared-init-lazy-scalar-mutation"));
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    let work = diagnostics.work;
    assert_eq!(work["postgres.annotation.shared_init.disabled"], 0);
    assert_eq!(work["postgres.annotation.shared_init.builds"], 3);
    assert_eq!(
        work["postgres.annotation.shared_init.cache_hits"], 1,
        "root-b only touches middle.version and must not preinstall lazy or state: {work:?}"
    );
}
