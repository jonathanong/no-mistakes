use super::*;

fn helper_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports-imported-helpers",
    ))
}

fn assert_helper_mock_outcomes(findings: &[RuleFinding]) {
    let targets = findings
        .iter()
        .filter_map(|finding| finding.target.as_deref())
        .collect::<HashSet<_>>();
    // A separate test without the imported helper and a test whose helper is
    // behind a mocked bridge still need these same-source findings.
    for unmocked in [
        "src/covered-leaf.mts",
        "src/bridge-leaf.mts",
        "src/factory-leaf.mts",
        "src/unimported-leaf.mts",
        "src/type-only-leaf.mts",
        "src/lazy-leaf.mts",
    ] {
        assert!(targets.contains(unmocked), "{unmocked}: {findings:?}");
    }
}

#[test]
fn statically_imported_nested_helper_mocks_cover_only_reachable_targets() {
    let root = helper_fixture();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config, None).unwrap();
    assert_helper_mock_outcomes(&findings);
}

#[test]
fn prepared_helper_mocks_match_standalone_and_use_shared_facts() {
    let root = helper_fixture();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            imports: true,
            dynamic_imports: true,
            source: true,
            ..Default::default()
        },
    );
    let findings = check_with_facts(&root, &config, None, &facts).unwrap();
    assert_helper_mock_outcomes(&findings);
}
