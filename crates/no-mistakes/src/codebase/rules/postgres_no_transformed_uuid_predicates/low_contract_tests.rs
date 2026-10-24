use super::*;

#[test]
fn low_reports_each_unknown_owner_even_without_a_verdict_sink() {
    let root = super::tests::fixture();
    let mut config = super::tests::config("");
    config.effort = Some(crate::config::v2::Effort::Low);
    let files = [root.join("verdict-inputs.ts"), root.join("schema.json")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 6, "{findings:#?}");
    assert_eq!(
        findings
            .iter()
            .filter(|row| row.target.as_deref() == Some("unanalyzable"))
            .count(),
        4
    );
}

#[test]
fn low_rejects_an_incomplete_prepared_projection_without_repairing_it() {
    let mut config = super::tests::config("");
    config.effort = Some(crate::config::v2::Effort::Low);
    let (config, files, sources, mut facts) = super::cache_tests::prepare("embedded.ts", config);
    let path = super::tests::fixture().join("embedded.ts");
    let ts = facts.ts.get(&path).unwrap().ts.clone();
    facts.ts.insert(
        path,
        std::sync::Arc::new(crate::codebase::check_facts::CheckFileFacts {
            ts,
            ..Default::default()
        }),
    );
    let reads = sources.physical_read_count();
    let error = check_with_files_sources_and_facts(
        &super::tests::fixture(),
        &config,
        &files,
        &sources,
        &facts,
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("prepared low-effort SQL syntax projection missing"));
    assert_eq!(sources.physical_read_count(), reads);
}
