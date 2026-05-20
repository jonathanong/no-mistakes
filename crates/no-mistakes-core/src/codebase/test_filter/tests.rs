use super::*;

#[test]
fn configured_suite_excludes_override_includes() {
    let root = fixture_root();
    let config = load_config_fixture(&root, "suite-exclude");
    let filter = TestFileFilter::new(&root, &config);

    assert!(filter.is_match_rel("backend/api/users.test.mts"));
    assert!(!filter.is_match_rel("backend/api/users.mock.test.mts"));
    assert!(filter.is_match_rel("integration/api/users.mock.test.mts"));
}

fn fixture_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/codebase-analysis/test-filter")
}

fn load_config_fixture(root: &Path, name: &str) -> NoMistakesConfig {
    let path = root.join(format!("{name}.no-mistakes.yml"));
    let yaml = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&yaml).unwrap()
}
