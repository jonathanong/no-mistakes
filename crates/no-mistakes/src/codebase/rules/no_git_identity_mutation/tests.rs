use super::runner::{is_managed_runner, is_managed_runner_only, parse_runs_on_values};
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use globset::GlobSet;
use std::path::Path;

fn check_file(
    path: &Path,
    root: &Path,
    exclude_set: &GlobSet,
    cond_set: &GlobSet,
    patterns: &[Regex; 3],
) -> Vec<RuleFinding> {
    let sources = crate::codebase::rules::source_store_for_files(&[path.to_path_buf()]);
    check_file_with_sources(path, root, exclude_set, cond_set, patterns, &sources)
}

fn fixture(path: &str) -> std::path::PathBuf {
    crate::test_support::rule_fixture_root("no-git-identity-mutation").join(path)
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

fn case_root(case: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(&fixture(&format!("cases/{case}")))
}

fn run_on_source(case: &str) -> Vec<RuleFinding> {
    let root = case_root(case);
    let path = root.join("script.sh");
    check_file(
        &path,
        &root,
        &empty_globset(),
        &empty_globset(),
        &patterns(),
    )
}

fn conditionally_allowed(case: &str) -> Vec<RuleFinding> {
    let root = case_root(case);
    let path = root.join(".github/workflows/ci.yml");
    let mut cond_builder = globset::GlobSetBuilder::new();
    cond_builder.add(globset::Glob::new(".github/workflows/*.yml").unwrap());
    let cond_set = cond_builder.build().unwrap();
    check_file(&path, &root, &empty_globset(), &cond_set, &patterns())
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
fn readonly_config_lookup_fixture_produces_no_findings() {
    let root = fixture("readonly-pass");
    let config = config_with_options("{}");
    let files = vec![root.join("setup.sh")];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn managed_runner_mapping_fixture_produces_no_findings() {
    let root = fixture("mapping-managed-pass");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn shell_form_flagged() {
    let findings = run_on_source("shell-form");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 1);
}

#[test]
fn env_var_form_not_flagged() {
    let findings = run_on_source("env-var-form");
    assert!(findings.is_empty());
}

#[test]
fn git_config_email_flagged() {
    let findings = run_on_source("git-config-email");
    assert_eq!(findings.len(), 1);
}

#[test]
fn read_only_shell_forms_not_flagged() {
    let findings = run_on_source("read-only-shell");
    assert!(findings.is_empty());
}

#[test]
fn array_form_flagged() {
    let findings = run_on_source("array-form");
    assert_eq!(findings.len(), 1);
}

#[test]
fn read_only_array_form_not_flagged() {
    let findings = run_on_source("read-only-array");
    assert!(findings.is_empty());
}

#[test]
fn helper_form_flagged() {
    let findings = run_on_source("helper-form");
    assert_eq!(findings.len(), 1);
}

#[test]
fn read_only_helper_form_not_flagged() {
    let findings = run_on_source("read-only-helper");
    assert!(findings.is_empty());
}

#[test]
fn excluded_path_skipped() {
    let root = case_root("excluded-path");
    let path = root.join("scripts/setup.sh");
    let mut builder = globset::GlobSetBuilder::new();
    builder.add(globset::Glob::new("scripts/**").unwrap());
    let exclude_set = builder.build().unwrap();
    let findings = check_file(&path, &root, &exclude_set, &empty_globset(), &patterns());
    assert!(findings.is_empty());
}

#[test]
fn conditionally_allowed_workflow_skipped_if_managed_runners_only() {
    let findings = conditionally_allowed("managed-runners-only");
    assert!(
        findings.is_empty(),
        "managed-runner-only workflow should be skipped"
    );
}

#[test]
fn conditionally_allowed_workflow_not_skipped_if_self_hosted_runner() {
    let findings = conditionally_allowed("self-hosted-runner");
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
fn is_managed_runner_only_quoted_single() {
    assert!(is_managed_runner_only("runs-on: 'ubuntu-latest'\n"));
}

#[test]
fn is_managed_runner_only_quoted_double() {
    assert!(is_managed_runner_only("runs-on: \"ubuntu-latest\"\n"));
}

#[test]
fn is_managed_runner_only_bracketed_list() {
    assert!(is_managed_runner_only(
        "runs-on: [ubuntu-latest, windows-latest]\n"
    ));
}

#[test]
fn is_managed_runner_only_bracketed_list_with_self_hosted() {
    assert!(!is_managed_runner_only(
        "runs-on: [ubuntu-latest, self-hosted]\n"
    ));
}

#[test]
fn parse_runs_on_values_trims_comments_quotes_and_empty_parts() {
    assert_eq!(
        parse_runs_on_values("[\"ubuntu-latest\", '', 'windows-latest'] # hosted"),
        vec!["ubuntu-latest".to_string(), "windows-latest".to_string()]
    );
}

#[test]
fn conditionally_allowed_workflow_skipped_if_managed_runner_is_quoted() {
    let findings = conditionally_allowed("quoted-managed-runner");
    assert!(
        findings.is_empty(),
        "quoted managed runner workflow should be skipped"
    );
}

#[test]
fn check_standalone_produces_no_findings_for_empty_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let config = config_with_options("{}");
    let findings = check(tmp.path(), &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn build_exclude_globset_with_patterns_works() {
    // Exercises lines 61-63: Glob::new succeeds and builder.add is called.
    let patterns = vec![
        "scripts/**".to_string(),
        ".github/workflows/*.yml".to_string(),
    ];
    let globset = build_exclude_globset(&patterns);
    assert!(globset.is_match("scripts/setup.sh"));
    assert!(globset.is_match(".github/workflows/ci.yml"));
    assert!(!globset.is_match("src/index.ts"));
}

#[test]
fn check_with_exclude_paths_uses_build_exclude_globset() {
    // config option excludePaths exercises build_exclude_globset via scan()
    let root = case_root("exclude-paths");
    let script = root.join("scripts/setup.sh");
    let config = config_with_options("excludePaths: [\"scripts/**\"]");
    let findings = check_with_files(&root, &config, &[script]).unwrap();
    assert!(
        findings.is_empty(),
        "excluded path should produce no findings"
    );
}

#[test]
fn is_managed_runner_ubuntu_versioned() {
    assert!(is_managed_runner("ubuntu-22.04"));
    assert!(is_managed_runner("ubuntu-24.04"));
    assert!(is_managed_runner("ubuntu-24.04-arm"));
    assert!(is_managed_runner("ubuntu-slim"));
    assert!(is_managed_runner("ubuntu-22.04-slim"));
}

#[test]
fn is_managed_runner_macos_versioned() {
    assert!(is_managed_runner("macos-14"));
    assert!(is_managed_runner("macos-latest"));
    assert!(is_managed_runner("macos-13-xlarge"));
}

#[test]
fn is_managed_runner_windows_versioned() {
    assert!(is_managed_runner("windows-2022"));
    assert!(is_managed_runner("windows-2025"));
    assert!(is_managed_runner("windows-latest"));
}

#[test]
fn is_managed_runner_self_hosted_not_managed() {
    assert!(!is_managed_runner("self-hosted"));
    assert!(!is_managed_runner("arc-runner"));
    assert!(!is_managed_runner("custom-runner"));
}

#[test]
fn line_number_is_correct() {
    let findings = run_on_source("line-number");
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

#[test]
fn non_script_file_not_flagged() {
    let root = case_root("non-script");
    let path = root.join("README.md");
    let findings = check_file(
        &path,
        &root,
        &empty_globset(),
        &empty_globset(),
        &patterns(),
    );
    assert!(
        findings.is_empty(),
        "markdown file should not be flagged; got {findings:?}"
    );
}

#[test]
fn commented_line_not_flagged() {
    let findings = run_on_source("commented-line");
    assert!(
        findings.is_empty(),
        "commented git config should not be flagged"
    );
}

#[test]
fn echo_line_not_flagged() {
    let findings = run_on_source("echo-line");
    assert!(
        findings.is_empty(),
        "echoed git config should not be flagged"
    );
}

#[test]
fn has_shell_shebang_bash() {
    assert!(has_shell_shebang("#!/bin/bash\necho hi\n"));
    assert!(has_shell_shebang("#!/bin/sh\necho hi\n"));
    assert!(has_shell_shebang("#!/usr/bin/env bash\necho hi\n"));
    assert!(has_shell_shebang("#!/usr/bin/env sh\necho hi\n"));
}

#[test]
fn has_shell_shebang_none() {
    assert!(!has_shell_shebang("echo hi\n"));
    assert!(!has_shell_shebang(""));
}

#[test]
fn shebang_file_without_sh_extension_flagged() {
    let root = case_root("shebang-deploy");
    let path = root.join("deploy");
    let findings = check_file(
        &path,
        &root,
        &empty_globset(),
        &empty_globset(),
        &patterns(),
    );
    assert_eq!(
        findings.len(),
        1,
        "shebang file should be flagged; got {findings:?}"
    );
}

#[test]
fn check_respects_target_roots() {
    let root = case_root("target-roots");
    let script = root.join("scripts/setup.sh");
    // Config with repository scope — check() should apply target_roots
    let config = config_with_options("{}");
    let files = vec![script];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert!(!findings.is_empty(), "should find violation in script");
}

#[test]
fn line_continued_git_config_is_flagged() {
    // Exercises EW60T: shell regex now allows \<newline> line continuations.
    let findings = run_on_source("line-continued");
    assert_eq!(
        findings.len(),
        1,
        "line-continued git config user.name should be flagged"
    );
}
