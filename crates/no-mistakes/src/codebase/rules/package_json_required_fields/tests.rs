use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/package-json-required-fields/fixture")
            .join(name),
    )
}

fn config_yaml() -> &'static str {
    r#"
private: true
type: module
license: UNLICENSED
requireScopedName: true
unscopedNameExceptions: [web]
mainWhenFileExists: index.mts
"#
}

fn config() -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(config_yaml()).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn run(root: &Path) -> Vec<RuleFinding> {
    let files = vec![
        root.join("packages/foo/package.json"),
        root.join("packages/foo/index.mts"),
    ];
    check_with_files(root, &config(), &files).unwrap()
}

#[test]
fn flags_shape_violations() {
    let findings = run(&fixture("fail"));
    let targets: Vec<_> = findings
        .iter()
        .filter_map(|finding| finding.target.as_deref())
        .collect();
    assert!(targets.contains(&"name"), "{findings:?}");
    assert!(targets.contains(&"private"), "{findings:?}");
    assert!(targets.contains(&"type"), "{findings:?}");
    assert!(targets.contains(&"license"), "{findings:?}");
    assert!(targets.contains(&"main"), "{findings:?}");
}

#[test]
fn pass_fixture_is_clean() {
    assert!(run(&fixture("pass")).is_empty());
}

#[test]
fn unscoped_exception_skips_name_check() {
    let root = fixture("exception");
    let files = vec![root.join("packages/web/package.json")];
    let findings = check_with_files(&root, &config(), &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn empty_options_report_nothing() {
    let root = fixture("fail");
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            ..Default::default()
        }],
        ..Default::default()
    };
    let files = vec![root.join("packages/foo/package.json")];
    assert!(check_with_files(&root, &config, &files).unwrap().is_empty());
}

#[test]
fn include_filter_still_sees_companion_files() {
    let root = fixture("fail");
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            include: vec!["**/package.json".to_string()],
            options: serde_yaml::from_str("mainWhenFileExists: index.mts\n").unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let files = vec![
        root.join("packages/foo/package.json"),
        root.join("packages/foo/index.mts"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.target.as_deref() == Some("main")),
        "{findings:?}"
    );
}

#[test]
fn invalid_json_is_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let pkg = dir.path().join("package.json");
    std::fs::write(&pkg, "{").unwrap();
    let findings = check_with_files(dir.path(), &config(), &[pkg]).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn nested_companion_path_is_resolved_from_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let root = crate::codebase::ts_resolver::normalize_path(dir.path());
    let pkg = root.join("package.json");
    let companion = root.join("src/index.js");
    std::fs::create_dir(companion.parent().unwrap()).unwrap();
    std::fs::write(&pkg, r#"{"name":"@scope/foo"}"#).unwrap();
    std::fs::write(&companion, "export {}\n").unwrap();
    let config = NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("mainWhenFileExists: src/index.js\n").unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let findings = check_with_files(&root, &config, &[pkg, companion]).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.target.as_deref() == Some("main")),
        "{findings:?}"
    );
}
