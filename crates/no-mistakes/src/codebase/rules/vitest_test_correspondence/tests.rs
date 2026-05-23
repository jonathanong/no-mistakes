use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn config_with_rule(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn fixture_root(subpath: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/vitest-test-correspondence")
            .join(subpath),
    )
}

#[test]
fn pass_fixture_has_no_findings() {
    let root = fixture_root("pass");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        findings.is_empty(),
        "expected no findings, got: {findings:?}"
    );
}

#[test]
fn fail_fixture_has_findings() {
    let root = fixture_root("fail");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(
        !findings.is_empty(),
        "expected findings for orphan test file"
    );
}

#[test]
fn test_file_with_corresponding_source_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend/mod")).unwrap();
    std::fs::write(root.join("backend/mod/index.mts"), "export {};\n").unwrap();
    std::fs::write(
        root.join("backend/mod/index.test.mts"),
        "test('x', () => {});\n",
    )
    .unwrap();
    let config = config_with_rule(
        "{scopes: [backend], testExtensions: [\".test.mts\"], testsDir: __tests__}",
    );
    let findings = check(root, &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn test_file_without_source_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend/mod")).unwrap();
    std::fs::write(
        root.join("backend/mod/index.test.mts"),
        "test('x', () => {});\n",
    )
    .unwrap();
    let config = config_with_rule(
        "{scopes: [backend], testExtensions: [\".test.mts\"], testsDir: __tests__}",
    );
    let findings = check(root, &config).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("no corresponding source file"));
}

#[test]
fn test_file_in_tests_dir_is_exempt() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend/__tests__")).unwrap();
    std::fs::write(
        root.join("backend/__tests__/index.test.mts"),
        "test('x', () => {});\n",
    )
    .unwrap();
    let config = config_with_rule(
        "{scopes: [backend], testExtensions: [\".test.mts\"], testsDir: __tests__}",
    );
    let findings = check(root, &config).unwrap();
    assert!(findings.is_empty(), "tests dir files should be exempt");
}

#[test]
fn out_of_scope_test_file_is_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("frontend/mod")).unwrap();
    std::fs::write(
        root.join("frontend/mod/index.test.mts"),
        "test('x', () => {});\n",
    )
    .unwrap();
    let config = config_with_rule(
        "{scopes: [backend], testExtensions: [\".test.mts\"], testsDir: __tests__}",
    );
    let findings = check(root, &config).unwrap();
    assert!(findings.is_empty(), "out-of-scope files should be ignored");
}

#[test]
fn check_with_files_works() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::create_dir_all(root.join("backend/mod")).unwrap();
    let test_file = root.join("backend/mod/index.test.mts");
    std::fs::write(&test_file, "test('x', () => {});\n").unwrap();
    let config = config_with_rule(
        "{scopes: [backend], testExtensions: [\".test.mts\"], testsDir: __tests__}",
    );
    let findings = check_with_files(root, &config, &[test_file]).unwrap();
    assert_eq!(findings.len(), 1);
}

#[test]
fn ts_tsx_test_file_candidates() {
    let candidates = source_candidates("src/mod", "widget", ".test.ts");
    assert!(candidates.contains(&"src/mod/widget.ts".to_string()));
    assert!(candidates.contains(&"src/mod/widget.tsx".to_string()));
    assert!(candidates.contains(&"src/mod/index.ts".to_string()));
    assert!(candidates.contains(&"src/mod/index.tsx".to_string()));
}
