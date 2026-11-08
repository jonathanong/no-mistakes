use super::*;

#[test]
fn active_sql_audit_keeps_excluded_callers_and_imported_directives_broad() {
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            BROAD.with(|value| value.set(false));
        }
    }
    let saved = crate::test_support::materialize_saved_fixture(
        &fixture().join("../../sql-audit-scope/fixture"),
    );
    let root = crate::codebase::ts_resolver::normalize_path(saved.path());
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let plan = prepared(&root, &files, &config);
    assert!(plan.postgres_sql_audit);
    assert!(!eligible(&plan, None));
    let mut merged = CheckFactPlan::default();
    merged.include(plan);
    assert!(merged.postgres_sql_audit);
    let run = |broad| {
        BROAD.with(|value| value.set(broad));
        let _restore = Restore;
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let report = crate::diagnostics::with_observer(Some(observer.clone()), || {
            crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                serde_json::json!({"root":root}),
            ))
            .unwrap()
        });
        let count = observer
            .snapshot()
            .work
            .get("postgres.embedded.collections")
            .copied()
            .unwrap_or(0);
        (
            serde_json::from_str::<serde_json::Value>(&report).unwrap(),
            count,
        )
    };
    let (full, full_count) = run(true);
    let (actual, count) = run(false);
    assert_eq!(actual, full);
    assert_eq!((count, full_count), (4, 4));
    let audit = actual["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "unused-disable-directive")
        .collect::<Vec<_>>();
    assert_eq!(audit.len(), 1, "{actual:#}");
    assert_eq!(audit[0]["file"], "helpers/directive.mts");
    assert_eq!(audit[0]["line"], 3);
    assert_eq!(audit[0]["target"], "postgres-no-offset");
}
