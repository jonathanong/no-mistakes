#[test]
fn configured_types_and_factory_members_preserve_exact_binding_provenance() {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("typed-client-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    let findings: Vec<_> = findings
        .iter()
        .filter(|finding| finding.rule == super::super::RULE_ID)
        .collect();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.file.as_str(), finding.line))
            .collect::<Vec<_>>(),
        [
            ("src/advanced.mts", 10),
            ("src/advanced.mts", 13),
            ("src/advanced.mts", 16),
            ("src/advanced.mts", 20),
            ("src/advanced.mts", 24),
            ("src/advanced.mts", 26),
            ("src/entry.mts", 7),
            ("src/entry.mts", 11),
            ("src/entry.mts", 14),
            ("src/entry.mts", 20),
            ("src/entry.mts", 22),
            ("src/forwarded.mts", 6),
            ("src/forwarded.mts", 13),
            ("src/forwarded.mts", 21),
            ("src/forwarded.mts", 28),
            ("src/reexports.mts", 4)
        ],
        "{findings:#?}"
    );
    assert!(findings
        .iter()
        .all(|finding| finding.message.contains("transaction client only")));
}

#[test]
fn configured_provenance_is_opt_in_and_rejects_malformed_selectors() {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("typed-client-fixture");
    assert!(
        crate::codebase::rules::run_check(&root, Some(&root.join("disabled.yml")), None)
            .unwrap()
            .is_empty()
    );
    for (file, message) in [
        ("bad-type.yml", "typedExecutors require"),
        ("bad-member.yml", "returnedMembers require"),
        ("bad-selector.yml", "unknown field"),
    ] {
        let error =
            crate::codebase::rules::run_check(&root, Some(&root.join(file)), None).unwrap_err();
        assert!(format!("{error:#}").contains(message), "{error:#}");
    }
}

#[test]
fn factory_metadata_preserves_existing_value_flow_operations() {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("typed-client-compat-fixture");
    let report = crate::effects_query::run(
        &root,
        None,
        None,
        "postgres",
        std::path::Path::new("src/entry.mts"),
        &[],
        None,
    )
    .unwrap();
    assert_eq!(
        report
            .call_sites
            .iter()
            .filter(|site| site.file == "src/entry.mts")
            .map(|site| (site.line, site.callee.as_str()))
            .collect::<Vec<_>>(),
        [
            (3, "primary.query"),
            (5, "primary.query"),
            (8, "primary.query"),
            (10, "primary.query"),
        ],
        "{report:#?}",
    );
    let baseline = crate::effects_query::run(
        &root,
        Some(&root.join("baseline.yml")),
        None,
        "postgres",
        std::path::Path::new("src/entry.mts"),
        &[],
        None,
    )
    .unwrap();
    assert_eq!(
        report, baseline,
        "metadata must preserve legacy report fields"
    );
}
