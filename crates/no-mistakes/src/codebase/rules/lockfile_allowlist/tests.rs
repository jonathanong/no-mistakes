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
            .join("../../test-cases/rules/lockfile-allowlist/fixture")
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
    assert!(!findings.is_empty(), "expected findings for yarn.lock");
    assert!(findings[0].message.contains("not allowed"));
}

#[test]
fn allowed_lockfile_passes() {
    let root = fixture_root("allowed-pnpm");
    let config = config_with_rule(
        "allowed: [pnpm-lock.yaml]\nbannedBasenames: [package-lock.json, yarn.lock]",
    );
    let findings = check(&root, &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn banned_lockfile_flagged() {
    let root = fixture_root("banned-yarn");
    let config = config_with_rule(
        "allowed: [pnpm-lock.yaml]\nbannedBasenames: [package-lock.json, yarn.lock]",
    );
    let findings = check(&root, &config).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].file.contains("yarn.lock"));
}

#[test]
fn default_banned_basenames_includes_npm() {
    let root = fixture_root("default-banned-npm");
    let config = config_with_rule("{}");
    let findings = check(&root, &config).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].file.contains("package-lock.json"));
}

#[test]
fn glob_pattern_in_allowed() {
    let root = fixture_root("glob-nested");
    let config =
        config_with_rule("allowed: [\"**/pnpm-lock.yaml\"]\nbannedBasenames: [pnpm-lock.yaml]");
    let findings = check(&root, &config).unwrap();
    assert!(
        findings.is_empty(),
        "glob should match nested pnpm-lock.yaml"
    );
}

#[test]
fn check_with_files_works() {
    let root = fixture_root("banned-yarn");
    let path = root.join("yarn.lock");
    let config = config_with_rule("allowed: [pnpm-lock.yaml]\nbannedBasenames: [yarn.lock]");
    let findings = check_with_files(&root, &config, &[path]).unwrap();
    assert_eq!(findings.len(), 1);
}

#[test]
fn invalid_allowed_glob_fails_closed() {
    let root = fixture_root("invalid-allowed-glob");
    let error = check(
        &root,
        &config_with_rule("allowed: ['[']\nbannedBasenames: [yarn.lock]"),
    )
    .expect_err("invalid allowlist glob must fail");
    assert!(
        error.to_string().contains("error parsing glob")
            || error.to_string().contains("unclosed")
            || error.to_string().contains("["),
        "{error:#}"
    );
}

#[test]
fn custom_banned_basenames_ignore_default_npm_lockfile() {
    let root = fixture_root("custom-banned-basenames");
    let findings = check(
        &root,
        &config_with_rule("allowed: [pnpm-lock.yaml]\nbannedBasenames: [yarn.lock]"),
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}
