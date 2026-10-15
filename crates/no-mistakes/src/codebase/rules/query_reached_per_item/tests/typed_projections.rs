fn projections_fixture() -> std::path::PathBuf {
    crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("typed-executor-projections-fixture")
}

#[test]
fn precise_type_projections_and_nullish_executors_reach_each_per_item_loop() {
    let findings = crate::codebase::rules::run_check(&projections_fixture(), None, None).unwrap();
    let actual = findings
        .iter()
        .filter(|finding| finding.rule == super::super::RULE_ID && finding.file == "entry.mts")
        .map(|finding| (finding.file.as_str(), finding.line))
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            ("entry.mts", 4),
            ("entry.mts", 7),
            ("entry.mts", 15),
            ("entry.mts", 20)
        ],
        "{findings:#?}"
    );
}

#[test]
fn effects_query_uses_the_same_exact_type_projection_and_alternative_provenance() {
    let root = projections_fixture();
    let report = crate::effects_query::run(
        &root,
        None,
        None,
        "postgres",
        std::path::Path::new("entry.mts"),
        &[],
        None,
    )
    .unwrap();
    assert_eq!(
        report
            .call_sites
            .iter()
            .filter(|site| site.file == "entry.mts")
            .map(|site| site.line)
            .collect::<Vec<_>>(),
        [4, 7, 15, 20],
        "{report:#?}"
    );
    for entry in [
        "controls.mts",
        "imported-utility.mts",
        "unselected-indexed.mts",
    ] {
        let report = crate::effects_query::run(
            &root,
            None,
            None,
            "postgres",
            std::path::Path::new(entry),
            &[],
            None,
        )
        .unwrap();
        assert!(report.call_sites.is_empty(), "{entry}: {report:#?}");
    }
}

#[test]
fn projected_annotations_keep_lexical_ownership_and_reconcile_callback_contexts() {
    let root = projections_fixture();
    for entry in [
        "controls.mts",
        "imported-utility.mts",
        "unselected-indexed.mts",
    ] {
        let report = crate::effects_query::run(
            &root,
            None,
            None,
            "postgres",
            std::path::Path::new(entry),
            &[],
            None,
        )
        .unwrap();
        assert!(report.call_sites.is_empty(), "{entry}: {report:#?}");
    }
}
