use super::config_project;
use std::path::Path;

fn project(source: &str, visible: &[&str]) -> crate::integration_tests::types::ConfigProject {
    let root = Path::new("/repo");
    let files = visible
        .iter()
        .map(|rel| root.join(rel))
        .collect::<crate::fx::PathSet>();
    config_project(root, "jest.config.js", root, source, Some(&files)).unwrap()
}

#[test]
fn test_match_literals_become_include_globs() {
    let parsed = project(
        r#"module.exports = { testMatch: ["**/*.test.ts", "<rootDir>/src/**/*.spec.ts"] };"#,
        &["src/value.test.ts"],
    );
    assert_eq!(
        parsed.include,
        vec!["**/*.test.ts".to_string(), "src/**/*.spec.ts".to_string()]
    );
    assert!(parsed.vitest_setup.is_empty());
    assert_eq!(parsed.config.as_deref(), Some("jest.config.js"));
}

#[test]
fn test_regex_matches_visible_files_into_include() {
    let parsed = project(
        r#"module.exports = { testRegex: "value.test.ts$" };"#,
        &["src/value.test.ts", "src/other.ts"],
    );
    assert_eq!(parsed.include, vec!["src/value.test.ts".to_string()]);
}

#[test]
fn empty_matchers_use_shared_vitest_jest_globs() {
    let parsed = project("module.exports = {};", &["src/value.test.ts"]);
    assert!(parsed
        .include
        .iter()
        .any(|pattern| pattern == "**/*.test.ts"));
    assert!(parsed.vitest_setup.is_empty());
}

#[test]
fn test_regex_and_test_match_are_unioned() {
    let parsed = project(
        r#"module.exports = { testMatch: ["**/*.spec.ts"], testRegex: "value.test.ts$" };"#,
        &["src/value.test.ts"],
    );
    assert!(parsed.include.contains(&"**/*.spec.ts".to_string()));
    assert!(parsed.include.contains(&"src/value.test.ts".to_string()));
}

#[test]
fn root_dir_and_relative_test_match_patterns_normalize() {
    let root = Path::new("/repo");
    let files = crate::fx::PathSet::default();
    let parsed = config_project(
        root,
        "jest.config.js",
        root,
        r#"module.exports = { testMatch: ["<rootDir>", "./src/**/*.test.ts"] };"#,
        Some(&files),
    )
    .unwrap();
    assert!(parsed
        .include
        .iter()
        .any(|pattern| pattern == "." || pattern.is_empty() || pattern == "/repo"));
    assert!(parsed
        .include
        .iter()
        .any(|pattern| pattern.ends_with("src/**/*.test.ts")));
}

#[test]
fn nested_config_sets_scope() {
    let root = Path::new("/repo");
    let config_dir = root.join("packages/app");
    let files = crate::fx::PathSet::default();
    let parsed = config_project(
        root,
        "packages/app/jest.config.js",
        &config_dir,
        r#"module.exports = { testMatch: ["**/*.test.ts"] };"#,
        Some(&files),
    )
    .unwrap();
    assert_eq!(parsed.scope.as_deref(), Some("packages/app"));
    assert_eq!(
        parsed.include,
        vec!["packages/app/**/*.test.ts".to_string()]
    );
}

#[test]
fn missing_visible_files_skips_test_regex_matches() {
    let root = Path::new("/repo");
    let parsed = config_project(
        root,
        "jest.config.js",
        root,
        r#"module.exports = { testRegex: "value\\.test\\.ts$" };"#,
        None,
    )
    .unwrap();
    assert!(parsed.include.is_empty());
}

#[test]
fn invalid_test_regex_is_an_error() {
    let root = Path::new("/repo");
    let files = crate::fx::PathSet::default();
    let error = config_project(
        root,
        "jest.config.js",
        root,
        r#"module.exports = { testRegex: "(" };"#,
        Some(&files),
    )
    .unwrap_err();
    assert!(error.to_string().contains("invalid Jest testRegex"));
}

#[test]
fn nested_root_dir_test_match_is_not_double_prefixed() {
    let root = Path::new("/repo");
    let config_dir = root.join("packages/app");
    let files = crate::fx::PathSet::default();
    let parsed = config_project(
        root,
        "packages/app/jest.config.js",
        &config_dir,
        r#"module.exports = { testMatch: ["<rootDir>/src/**/*.test.ts"] };"#,
        Some(&files),
    )
    .unwrap();
    assert_eq!(
        parsed.include,
        vec!["packages/app/src/**/*.test.ts".to_string()]
    );
}

#[test]
fn unmatched_test_regex_does_not_fall_back_to_default_globs() {
    let parsed = project(
        r#"module.exports = { testRegex: "integration/.*" };"#,
        &["src/value.test.ts"],
    );
    assert!(parsed.include.is_empty());
}

#[test]
fn quoted_json_test_match_keys_are_parsed() {
    let parsed = project(
        r#"{ "testMatch": ["integration/**/*.test.js"] }"#,
        &["src/value.test.ts"],
    );
    assert_eq!(parsed.include, vec!["integration/**/*.test.js".to_string()]);
}
