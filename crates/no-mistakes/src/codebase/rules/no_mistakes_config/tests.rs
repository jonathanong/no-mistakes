use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/no-mistakes-config/fixture")
            .join(name),
    )
}

fn enable(mut config: NoMistakesConfig) -> NoMistakesConfig {
    config.rules.insert(
        0,
        RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            ..Default::default()
        },
    );
    config
}

fn load(root: &Path) -> NoMistakesConfig {
    let yaml = std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap();
    enable(serde_yaml::from_str(&yaml).unwrap())
}

fn files(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![root.join(".no-mistakes.yml")];
    for name in [
        "web/src/index.ts",
        "vitest.config.ts",
        "playwright/tests/a.spec.mts",
    ] {
        let path = root.join(name);
        if path.exists() {
            files.push(path);
        }
    }
    files
}

fn run(name: &str) -> Vec<RuleFinding> {
    let root = fixture(name);
    check_with_files(&root, &load(&root), &files(&root)).unwrap()
}

#[test]
fn missing_project_root_is_a_finding() {
    let findings = run("missing-path");
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("projects.web.root")
                && finding.message.contains("missing")),
        "{findings:?}"
    );
}

#[test]
fn empty_exclude_glob_is_a_finding() {
    let findings = run("empty-glob");
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("matches no tracked files")),
        "{findings:?}"
    );
}

#[test]
fn env_level_limit_with_direct_group_is_a_finding() {
    let findings = run("env-limit");
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("#9440")),
        "{findings:?}"
    );
}

#[test]
fn omitted_groups_still_flag_env_level_limit() {
    let root = fixture("pass");
    let config = enable(
        serde_yaml::from_str(
            "testPlan:\n  vitest:\n    environments:\n      prePush:\n        limit:\n          files: 10\n",
        )
        .unwrap(),
    );
    let findings = check_with_files(&root, &config, &files(&root)).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("#9440")),
        "{findings:?}"
    );
}

#[test]
fn valid_config_is_silent() {
    assert!(run("pass").is_empty(), "{:?}", run("pass"));
}

#[test]
fn disabled_rule_is_silent() {
    let root = fixture("missing-path");
    let config: NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    assert!(check_with_files(&root, &config, &files(&root))
        .unwrap()
        .is_empty());
}
