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
fn playwright_project_excludes_do_not_suppress_vitest_fallback_tests() {
    let root = fixture_root();
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.projects.insert(
        "unit".to_string(),
        TestProjectPolicy {
            include: vec!["unit/**/*.spec.ts".to_string()],
            ..Default::default()
        },
    );
    config.tests.playwright.projects.insert(
        "chromium".to_string(),
        TestProjectPolicy {
            include: vec!["**/*.spec.ts".to_string()],
            exclude: vec!["unit/**".to_string()],
            ..Default::default()
        },
    );

    let filter = TestFileFilter::new(&root, &config);

    assert!(filter.is_match_rel("unit/foo.spec.ts"));
}

#[test]
fn configured_project_exclude_blocks_generic_fallback_outside_runner_heuristic() {
    let root = fixture_root();
    let mut config = NoMistakesConfig::default();
    config.tests.playwright.projects.insert(
        "chromium".to_string(),
        TestProjectPolicy {
            include: vec!["e2e/**/*.spec.ts".to_string()],
            exclude: vec!["e2e/flaky.spec.ts".to_string()],
            ..Default::default()
        },
    );

    let filter = TestFileFilter::new(&root, &config);

    assert!(filter.is_match_rel("e2e/home.spec.ts"));
    assert!(!filter.is_match_rel("e2e/flaky.spec.ts"));
}

#[test]
fn invalid_project_config_falls_back_to_default_test_matching() {
    let root = tempfile::tempdir().unwrap();
    let config = load_config_fixture(&fixture_root(), "missing-vite-config");
    let filter = TestFileFilter::new(root.path(), &config);

    assert!(filter.is_match_rel("web/example.test.ts"));
    assert!(!filter.is_match_rel("web/example.ts"));
}

#[test]
fn always_include_globs_surface_suite_excluded_mock_tests() {
    let root = fixture_root();
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("missing.vitest.config.mts".to_string()));
    config.tests.vitest.projects.insert(
        "api".to_string(),
        TestProjectPolicy {
            include: vec!["backend/api/**/*.test.mts".to_string()],
            exclude: vec!["backend/api/**/*.mock.test.mts".to_string()],
            ..Default::default()
        },
    );
    config.tests.impact.always_include_tests = vec!["**/*.mock.test.mts".to_string()];

    let filter = TestFileFilter::new(&root, &config);

    // The stub test is normally dropped by the suite exclude, but the
    // always-include glob surfaces it anyway.
    assert!(filter.is_match_rel("backend/api/users.mock.test.mts"));
    // Regular suite tests still match.
    assert!(filter.is_match_rel("backend/api/users.test.mts"));
    // A non-test file that matches no stub glob is still not a test.
    assert!(!filter.is_match_rel("backend/api/users.mts"));
}

#[test]
fn empty_always_include_globs_leave_suite_excludes_intact() {
    let root = fixture_root();
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("missing.vitest.config.mts".to_string()));
    config.tests.vitest.projects.insert(
        "api".to_string(),
        TestProjectPolicy {
            include: vec!["backend/api/**/*.test.mts".to_string()],
            exclude: vec!["backend/api/**/*.mock.test.mts".to_string()],
            ..Default::default()
        },
    );
    // No always-include config: behavior is unchanged from before this feature.
    let filter = TestFileFilter::new(&root, &config);

    assert!(!filter.is_match_rel("backend/api/users.mock.test.mts"));
}

#[test]
fn malformed_always_include_glob_does_not_panic() {
    let root = fixture_root();
    let mut config = NoMistakesConfig::default();
    config.tests.impact.always_include_tests = vec!["[".to_string()];

    let filter = TestFileFilter::new(&root, &config);

    // Bad glob degrades to no always-include; default fallback still applies.
    assert!(filter.is_match_rel("foo.test.mts"));
    assert!(!filter.is_match_rel("foo.mts"));
}

#[test]
fn malformed_always_include_glob_is_skipped_keeping_valid_ones() {
    let root = fixture_root();
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.projects.insert(
        "api".to_string(),
        TestProjectPolicy {
            include: vec!["backend/api/**/*.test.mts".to_string()],
            exclude: vec!["backend/api/**/*.mock.test.mts".to_string()],
            ..Default::default()
        },
    );
    // One malformed pattern is skipped; the valid one still surfaces the stub.
    config.tests.impact.always_include_tests =
        vec!["[".to_string(), "**/*.mock.test.mts".to_string()];

    let filter = TestFileFilter::new(&root, &config);

    assert!(filter.is_match_rel("backend/api/users.mock.test.mts"));
}

fn fixture_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/test-filter/fixture")
}

fn load_config_fixture(root: &Path, name: &str) -> NoMistakesConfig {
    let path = root.join(format!("{name}.no-mistakes.yml"));
    let yaml = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&yaml).unwrap()
}
