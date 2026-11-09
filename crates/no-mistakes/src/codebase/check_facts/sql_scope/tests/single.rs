use super::*;

#[test]
fn actual_single_sql_checks_bypass_full_scope_and_reuse_strict_subset_classification() {
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            BROAD.with(|value| value.set(false));
        }
    }
    let saved = crate::test_support::materialize_saved_fixture(&fixture());
    let root = crate::codebase::ts_resolver::normalize_path(saved.path());
    let run = |config: &str, broad| {
        BROAD.with(|value| value.set(broad));
        let _restore = Restore;
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let result = crate::diagnostics::with_observer(Some(observer.clone()), || {
            crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                serde_json::json!({"root":root, "config":config}),
            ))
            .unwrap()
        });
        (
            serde_json::from_str::<serde_json::Value>(&result).unwrap(),
            observer.snapshot().work,
        )
    };
    for (config, delegations, walks) in [("all-selected.yml", 0, 6), ("middle-exclude.yml", 1, 5)] {
        let (full, full_work) = run(config, true);
        let (actual, work) = run(config, false);
        assert_eq!(actual, full, "{config}");
        assert_eq!(
            work.get("sql_scope.single_delegations")
                .copied()
                .unwrap_or(0),
            delegations,
            "{config}"
        );
        assert_eq!(
            work.get("sql_scope.seed_checks"),
            Some(&6),
            "each source is classified once: {config}"
        );
        assert_eq!(
            work.get("postgres.embedded.collections"),
            Some(&walks),
            "{config}"
        );
        assert_eq!(
            full_work.get("postgres.embedded.collections"),
            Some(&6),
            "{config}"
        );
        assert_eq!(work.get("check.fact_collections"), Some(&1), "{config}");
        assert_eq!(
            full_work.get("check.fact_collections"),
            Some(&1),
            "{config}"
        );
    }
}
