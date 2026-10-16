#[test]
fn namespace_local_executor_class_does_not_acquire_imported_type() {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("namespace-shadow-fixture");
    let expected: std::collections::BTreeMap<String, Vec<usize>> =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected.json")).unwrap())
            .unwrap();
    for (entry, lines) in &expected {
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
        assert_eq!(
            report.call_sites.iter().map(|site| site.line).collect::<Vec<_>>(),
            *lines,
            "Local type declarations must not borrow an imported type identity: {entry}: {report:#?}",
        );
        assert!(report.call_sites.iter().all(|site| site.file == *entry));
    }
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    let mut wanted: Vec<_> = expected
        .iter()
        .flat_map(|(file, lines)| lines.iter().map(move |line| (file.as_str(), *line)))
        .collect();
    wanted.sort();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.file.as_str(), finding.line))
            .collect::<Vec<_>>(),
        wanted,
        "{findings:#?}"
    );
}
