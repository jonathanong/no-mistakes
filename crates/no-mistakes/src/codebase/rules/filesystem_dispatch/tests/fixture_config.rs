pub(super) fn all_rules_fixture(directory: &str) -> tempfile::TempDir {
    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/filesystem-dispatch")
        .join(directory);
    crate::test_support::materialize_saved_fixture(&source)
}

pub(super) fn prepared_rules(
    directory: &str,
    expected: &[&str],
) -> (tempfile::TempDir, std::path::PathBuf) {
    let fixture = all_rules_fixture(directory);
    let config_path = fixture.path().join(".no-mistakes.yml");
    let loaded = crate::config::v2::load_v2_config(fixture.path(), Some(&config_path)).unwrap();
    assert_eq!(
        loaded
            .rules
            .iter()
            .map(|rule| rule.rule.as_str())
            .collect::<Vec<_>>(),
        expected,
    );
    (fixture, config_path)
}
