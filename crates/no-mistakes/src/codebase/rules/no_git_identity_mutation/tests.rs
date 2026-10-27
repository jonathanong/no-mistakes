use super::runner::{is_managed_runner, is_managed_runner_only, parse_runs_on_values};
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use globset::GlobSet;
use std::path::Path;

// Output cases moved to cli_rra_snapshot_no_git_identity_mutation.

fn check_file(
    path: &Path,
    root: &Path,
    exclude_set: &GlobSet,
    cond_set: &GlobSet,
    patterns: &[regex::Regex; 3],
) -> Vec<RuleFinding> {
    let sources = crate::codebase::rules::source_store_for_files(&[path.to_path_buf()]);
    check_file_with_sources(path, root, exclude_set, cond_set, patterns, &sources)
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
