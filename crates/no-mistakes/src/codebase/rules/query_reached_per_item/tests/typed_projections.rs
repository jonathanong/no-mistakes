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
        "class-utility.mts",
        "unsupported-shapes.mts",
        "forward-utility.mts",
        "forward-import-utility.mts",
        "forward-type-shadow.mts",
        "contextual-controls.mts",
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
fn projected_annotations_reconcile_every_forwarded_callback_context() {
    let report = crate::effects_query::run(
        &projections_fixture(),
        None,
        None,
        "postgres",
        std::path::Path::new("contextual-agreement.mts"),
        &[],
        None,
    )
    .unwrap();
    assert_eq!(
        report
            .call_sites
            .iter()
            .map(|site| (site.line, site.callee.as_str()))
            .collect::<Vec<_>>(),
        [(17, "TransactionQuery")],
        "{report:#?}"
    );
}

#[test]
fn nullish_effect_candidates_keep_independent_transaction_and_type_evidence() {
    let root = projections_fixture();
    let report = crate::effects_query::run(
        &root,
        None,
        None,
        "postgres",
        std::path::Path::new("nullish-branches.mts"),
        &[],
        None,
    )
    .unwrap();
    assert_eq!(
        report
            .call_sites
            .iter()
            .map(|site| (site.line, site.callee.as_str()))
            .collect::<Vec<_>>(),
        [
            (7, "TransactionQuery"),
            (11, "write"),
            (15, "write"),
            (19, "TransactionQuery")
        ],
        "{report:#?}"
    );
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    let findings = findings
        .iter()
        .filter(|finding| finding.file == "nullish-branches.mts")
        .collect::<Vec<_>>();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (
                finding.line,
                finding.message.contains("transaction client only")
            ))
            .collect::<Vec<_>>(),
        [(7, true), (11, false), (15, false), (19, true)],
        "{findings:#?}"
    );
}

#[test]
fn projected_type_shapes_keep_object_members_and_explicit_owner_bindings() {
    let report = crate::effects_query::run(
        &projections_fixture(),
        None,
        None,
        "postgres",
        std::path::Path::new("type-shapes.mts"),
        &[],
        None,
    )
    .unwrap();
    assert_eq!(
        report
            .call_sites
            .iter()
            .map(|site| site.line)
            .collect::<Vec<_>>(),
        [8, 12, 15, 18, 22],
        "{report:#?}"
    );
}

#[test]
fn nullish_receipts_preserve_factory_member_bound_and_async_admission() {
    for (file, expected) in [
        ("nullish-receipts.mts", vec![6, 10, 14]),
        ("recursive-indexed-types.mts", vec![5, 10]),
        ("nullish-literals-and-types.mts", vec![5, 9, 14]),
        ("nullish-acquisitions.mts", vec![15, 26, 35]),
    ] {
        let report = crate::effects_query::run(
            &projections_fixture(),
            None,
            None,
            "postgres",
            std::path::Path::new(file),
            &[],
            None,
        )
        .unwrap();
        assert_eq!(
            report
                .call_sites
                .iter()
                .map(|site| site.line)
                .collect::<Vec<_>>(),
            expected,
            "{file}: {report:#?}"
        );
    }
}
