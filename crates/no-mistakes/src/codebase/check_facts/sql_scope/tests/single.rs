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

#[test]
fn noncanonical_single_inventories_keep_broad_collection_and_findings() {
    let saved = crate::test_support::materialize_saved_fixture(&fixture());
    let root = crate::codebase::ts_resolver::normalize_path(saved.path());
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let plan = prepared(&root, &files, &config);
    let mut unsorted = files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
        .cloned()
        .collect::<Vec<_>>();
    unsorted.sort();
    unsorted.swap(0, 1);
    assert!(seeds::single(&root, &unsorted, &plan).is_none());
    let mut duplicate = unsorted.clone();
    duplicate.sort();
    duplicate.insert(
        1,
        duplicate[0]
            .parent()
            .unwrap()
            .join(".")
            .join(duplicate[0].file_name().unwrap()),
    );
    assert_ne!(duplicate[0].as_os_str(), duplicate[1].as_os_str());
    assert!(seeds::single(&root, &duplicate, &plan).is_none());
    let mut broad = plan.clone();
    broad.embedded_sql = false;
    let (mut full, full_walks) = counts(|| vec![collect_single(&root, files.clone(), broad)]);
    let full = full.pop().unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let actual = crate::diagnostics::with_observer(Some(observer.clone()), || {
        collect_single(&root, unsorted, plan)
    });
    let work = observer.snapshot().work;
    assert_eq!(full_walks, 6);
    assert_eq!(work.get("postgres.embedded.collections"), Some(&6));
    assert_eq!(
        work.get("sql_scope.single_delegations")
            .copied()
            .unwrap_or(0),
        0
    );
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let actual_findings = findings(&root, &files, &sources, &config, &actual);
    assert_eq!(
        actual_findings,
        findings(&root, &files, &sources, &config, &full)
    );
    assert_eq!(actual_findings.len(), 1);
    assert_eq!(actual_findings[0].rule, "postgres-no-offset");
    assert_eq!(actual_findings[0].file, "src/caller.mts");
    assert!(actual_findings[0].message.contains("OFFSET"));
    let broken = root.join("unrelated/broken.mts");
    assert!(actual.ts[&broken].parse_error.is_some());
    assert_eq!(actual.ts[&broken].parse_error, full.ts[&broken].parse_error);
}
