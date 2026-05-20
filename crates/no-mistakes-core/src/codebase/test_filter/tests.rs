use super::*;
use crate::config::v2::schema::{StringOrList, TestProjectPolicy};
use std::collections::BTreeMap;

#[test]
fn configured_suite_excludes_override_includes() {
    let root = fixture_root();
    let config = load_config_fixture(&root, "suite-exclude");
    let filter = TestFileFilter::new(&root, &config);

    assert!(filter.is_match_rel("backend/api/users.test.mts"));
    assert!(!filter.is_match_rel("backend/api/users.mock.test.mts"));
    assert!(filter.is_match_rel("integration/api/users.mock.test.mts"));
}

#[test]
fn configured_suite_filters_skip_when_project_config_fails_to_load() {
    let root = fixture_root();
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("missing.vitest.config.mts".to_string()));
    config.tests.vitest.projects.insert(
        "unit".to_string(),
        TestProjectPolicy {
            integration_suites: BTreeMap::from([("openai".to_string(), Vec::new())]),
        },
    );

    let filter = TestFileFilter::new(&root, &config);

    assert!(filter.suites.is_empty());
}

fn fixture_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/codebase-analysis/test-filter")
}

fn load_config_fixture(root: &Path, name: &str) -> NoMistakesConfig {
    let path = root.join(format!("{name}.no-mistakes.yml"));
    let yaml = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&yaml).unwrap()
}
