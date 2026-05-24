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
fn configured_suite_filters_keep_explicit_globs_when_project_config_fails_to_load() {
    let root = fixture_root();
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("missing.vitest.config.mts".to_string()));
    config.tests.vitest.projects.insert(
        "api".to_string(),
        TestProjectPolicy {
            include: vec!["backend/api/**/*.test.mts".to_string()],
            exclude: vec!["backend/api/**/*.mock.test.mts".to_string()],
            integration_suites: BTreeMap::from([(
                "openai".to_string(),
                vec!["openai".to_string()],
            )]),
        },
    );
    config.tests.vitest.projects.insert(
        "unit".to_string(),
        TestProjectPolicy {
            integration_suites: BTreeMap::from([("openai".to_string(), Vec::new())]),
            ..Default::default()
        },
    );

    let filter = TestFileFilter::new(&root, &config);

    assert_eq!(filter.suites.len(), 1);
    assert!(filter.is_match_rel("backend/api/users.test.mts"));
    assert!(!filter.is_match_rel("backend/api/users.mock.test.mts"));
    assert!(filter.is_match_rel("backend/unit.test.mts"));
}

#[test]
fn configured_suite_filters_use_explicit_globs_without_loading_project_config() {
    let root = fixture_root();
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("missing.vitest.config.mts".to_string()));
    config.tests.vitest.projects.insert(
        "api".to_string(),
        TestProjectPolicy {
            include: vec!["backend/api/**/*.test.mts".to_string()],
            exclude: vec!["backend/api/**/*.mock.test.mts".to_string()],
            integration_suites: BTreeMap::from([(
                "openai".to_string(),
                vec!["openai".to_string()],
            )]),
        },
    );

    let filter = TestFileFilter::new(&root, &config);

    assert_eq!(filter.suites.len(), 1);
    assert!(filter.is_match_rel("backend/api/users.test.mts"));
    assert!(!filter.is_match_rel("backend/api/users.mock.test.mts"));
}

#[test]
fn invalid_project_config_falls_back_to_default_test_matching() {
    let root = tempfile::tempdir().unwrap();
    let config = load_config_fixture(&fixture_root(), "missing-vite-config");
    let filter = TestFileFilter::new(root.path(), &config);

    assert!(filter.is_match_rel("web/example.test.ts"));
    assert!(!filter.is_match_rel("web/example.ts"));
}

fn fixture_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/codebase-analysis/test-filter")
}

fn load_config_fixture(root: &Path, name: &str) -> NoMistakesConfig {
    let path = root.join(format!("{name}.no-mistakes.yml"));
    let yaml = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&yaml).unwrap()
}
