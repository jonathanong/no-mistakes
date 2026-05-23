use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use globset::GlobSet;
use std::path::Path;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/no-git-identity-mutation")
        .join(path)
}

fn config_with_options(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

fn empty_globset() -> GlobSet {
    GlobSet::empty()
}

fn patterns() -> [regex::Regex; 3] {
    build_patterns()
}

fn run_on_source(source: &str) -> Vec<RuleFinding> {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("script.sh");
    std::fs::write(&path, source).unwrap();
    check_file(
        &path,
        tmp.path(),
        &empty_globset(),
        &empty_globset(),
        &patterns(),
    )
}

#[test]
fn pass_fixture_produces_no_findings() {
    let root = fixture("pass");
    let config = config_with_options("{}");
    let files = vec![root.join("setup.sh")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn fail_fixture_produces_findings() {
    let root = fixture("fail");
    let config = config_with_options("{}");
    let files = vec![root.join("setup.sh")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(!findings.is_empty(), "expected findings");
    assert!(
        findings
            .iter()
            .all(|f| f.message.contains("git config user")),
        "message should mention git config user"
    );
}

#[test]
fn shell_form_flagged() {
    let findings = run_on_source("git config user.name \"Bot\"\n");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 1);
}

#[test]
fn env_var_form_not_flagged() {
    let findings = run_on_source("export GIT_AUTHOR_NAME=\"Bot\"\ngit commit -m 'chore'\n");
    assert!(findings.is_empty());
}

#[test]
fn git_config_email_flagged() {
    let findings = run_on_source("git config user.email \"bot@example.com\"\n");
    assert_eq!(findings.len(), 1);
}

#[test]
fn array_form_flagged() {
    let findings = run_on_source("exec('git', 'config', 'user.name', 'Bot')\n");
    assert_eq!(findings.len(), 1);
}

#[test]
fn helper_form_flagged() {
    let findings = run_on_source("git('config', 'user.name', 'Bot')\n");
    assert_eq!(findings.len(), 1);
}

#[test]
fn excluded_path_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("scripts").join("setup.sh");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "git config user.name \"Bot\"\n").unwrap();
    let mut builder = globset::GlobSetBuilder::new();
    builder.add(globset::Glob::new("scripts/**").unwrap());
    let exclude_set = builder.build().unwrap();
    let findings = check_file(
        &path,
        tmp.path(),
        &exclude_set,
        &empty_globset(),
        &patterns(),
    );
    assert!(findings.is_empty());
}

#[test]
fn conditionally_allowed_workflow_skipped_if_managed_runners_only() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join(".github/workflows")).unwrap();
    let content = "runs-on: ubuntu-latest\ngit config user.name \"Bot\"\n";
    let path = tmp.path().join(".github/workflows/ci.yml");
    std::fs::write(&path, content).unwrap();
    let mut cond_builder = globset::GlobSetBuilder::new();
    cond_builder.add(globset::Glob::new(".github/workflows/*.yml").unwrap());
    let cond_set = cond_builder.build().unwrap();
    let findings = check_file(&path, tmp.path(), &empty_globset(), &cond_set, &patterns());
    assert!(
        findings.is_empty(),
        "managed-runner-only workflow should be skipped"
    );
}

#[test]
fn conditionally_allowed_workflow_not_skipped_if_self_hosted_runner() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join(".github/workflows")).unwrap();
    let content = "runs-on: self-hosted\ngit config user.name \"Bot\"\n";
    let path = tmp.path().join(".github/workflows/ci.yml");
    std::fs::write(&path, content).unwrap();
    let mut cond_builder = globset::GlobSetBuilder::new();
    cond_builder.add(globset::Glob::new(".github/workflows/*.yml").unwrap());
    let cond_set = cond_builder.build().unwrap();
    let findings = check_file(&path, tmp.path(), &empty_globset(), &cond_set, &patterns());
    assert!(
        !findings.is_empty(),
        "self-hosted runner workflow should not be skipped"
    );
}

#[test]
fn is_managed_runner_only_all_managed() {
    let content = "runs-on: ubuntu-latest\nruns-on: macos-latest\n";
    assert!(is_managed_runner_only(content));
}

#[test]
fn is_managed_runner_only_mixed() {
    let content = "runs-on: ubuntu-latest\nruns-on: self-hosted\n";
    assert!(!is_managed_runner_only(content));
}

#[test]
fn is_managed_runner_only_none() {
    assert!(!is_managed_runner_only("no runners here\n"));
}

#[test]
fn line_number_is_correct() {
    let findings = run_on_source("#!/bin/bash\n# comment\ngit config user.name \"Bot\"\n");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 3);
}

#[test]
fn unreadable_file_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("nonexistent.sh");
    let findings = check_file(
        &path,
        tmp.path(),
        &empty_globset(),
        &empty_globset(),
        &patterns(),
    );
    assert!(findings.is_empty());
}
