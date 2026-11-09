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
    assert!(plan
        .embedded_sql_rule_ids
        .values()
        .flatten()
        .all(|scan| scan.suppression_audit));
    assert!(!eligible(&plan, None));
    let mut merged = CheckFactPlan::default();
    merged.include(plan);
    assert!(!eligible(&merged, None));
    let mut ignored_config = config.clone();
    ignored_config.report_unused_disable_directives =
        crate::config::v2::DisableDirectivePolicy::Ignore;
    let ignored = prepared(&root, &files, &ignored_config);
    assert!(eligible(&ignored, None));
    let scan_count = ignored.embedded_sql_rule_ids.values().flatten().count();
    for (left, right) in [(merged.clone(), ignored.clone()), (ignored, merged)] {
        let mut combined = left;
        combined.include(right);
        assert!(!eligible(&combined, None));
        assert_eq!(
            combined.embedded_sql_rule_ids.values().flatten().count(),
            scan_count
        );
        assert!(combined
            .embedded_sql_rule_ids
            .values()
            .flatten()
            .any(|scan| scan.suppression_audit));
    }
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

#[test]
fn non_sql_audit_demand_does_not_widen_unaudited_sql_profiles() {
    let saved = crate::test_support::materialize_saved_fixture(&fixture());
    let root = crate::codebase::ts_resolver::normalize_path(saved.path());
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sql_config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let no_sql_config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("no-sql.yml"))).unwrap();
    assert!(no_sql_config.audits_disable_directives());
    let sql = prepared(&root, &files, &sql_config);
    let no_sql = prepared(&root, &files, &no_sql_config);
    assert!(no_sql.embedded_sql_rule_ids.is_empty());
    assert!(!no_sql.embedded_sql);
    let (_, walks) = counts(|| vec![collect_single(&root, files.clone(), no_sql.clone())]);
    assert_eq!(walks, 0);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let mut broad = sql.clone();
    broad.embedded_sql = false;
    let (mut full, full_walks) = counts(|| vec![collect_single(&root, files.clone(), broad)]);
    assert_eq!(full_walks, 6);
    let expected = findings(&root, &files, &sources, &sql_config, &full.pop().unwrap());
    assert_eq!(expected.len(), 1);
    for (left, right) in [(sql.clone(), no_sql.clone()), (no_sql, sql)] {
        let mut merged = left;
        merged.include(right);
        assert!(eligible(&merged, None));
        let (mut maps, walks) = counts(|| vec![collect_single(&root, files.clone(), merged)]);
        assert_eq!(walks, 4);
        assert_eq!(
            findings(&root, &files, &sources, &sql_config, &maps.pop().unwrap()),
            expected
        );
    }
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let report = crate::diagnostics::with_observer(Some(observer.clone()), || {
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({"root":root,"config":root.join("no-sql.yml")}),
        ))
        .unwrap()
    });
    let report: serde_json::Value = serde_json::from_str(&report).unwrap();
    assert_eq!(report["rules"].as_array().unwrap().len(), 1, "{report:#}");
    assert_eq!(report["rules"][0]["rule"], "forbidden-dependencies");
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            BROAD.with(|value| value.set(self.0));
        }
    }
    let _restore = Restore(BROAD.with(|value| value.replace(true)));
    let full = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({"root":root,"config":root.join("no-sql.yml")}),
    ))
    .unwrap();
    assert_eq!(
        report,
        serde_json::from_str::<serde_json::Value>(&full).unwrap()
    );
    assert_eq!(
        observer
            .snapshot()
            .work
            .get("postgres.embedded.collections")
            .copied()
            .unwrap_or(0),
        0
    );
}
