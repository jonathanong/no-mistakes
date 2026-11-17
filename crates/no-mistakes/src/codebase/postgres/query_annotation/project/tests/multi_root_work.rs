use super::demand_followups::observed_check_with_config;

#[test]
fn saved_multi_root_walks_preserve_named_queries_and_prune_unrelated_roots() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-multi-root-shared-walks");
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected.json")).unwrap())
            .unwrap();
    let mut previous = None;
    for effort in ["low", "max"] {
        let (report, diagnostics) =
            observed_check_with_config(&root, Some(root.join(format!("{effort}.yml"))));
        assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
        assert_eq!(report["advisories"], serde_json::json!([]), "{report}");
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 1, "{report}");
        assert_eq!(findings[0]["rule"], "postgres-require-query-annotation");
        assert_eq!(findings[0]["file"], expected["finding"]["file"]);
        assert_eq!(findings[0]["line"], expected["finding"]["line"]);
        if effort == "max" {
            let roots = expected["roots"].as_u64().unwrap();
            let excluded = expected["excludedRoots"].as_u64().unwrap();
            #[cfg(feature = "test-instrumentation")]
            {
                let (off, disabled) =
                    crate::benchmark_support::check_json_observed(&root, false).unwrap();
                assert!(disabled.work.is_empty());
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(&off).unwrap(),
                    report
                );
            }
            let work = diagnostics.work;
            let skipped = work["postgres.annotation.prefix_unchanged_binding_skips"];
            let walks = work["postgres.annotation.prefix_walks"];
            assert!(skipped > 0, "{work:?}");
            assert!(walks <= skipped, "{work:?}");
            assert_eq!(work["postgres.annotation.roots"], roots);
            assert_eq!(work["postgres.annotation.roots_after_scoping"], roots);
            assert_eq!(work["postgres.annotation.excluded_roots_pruned"], excluded);
            assert_eq!(work["postgres.annotation.excluded_roots_retained"], 0);
        }
        if let Some(previous) = previous.replace(report.clone()) {
            assert_eq!(report, previous);
        }
    }
}
