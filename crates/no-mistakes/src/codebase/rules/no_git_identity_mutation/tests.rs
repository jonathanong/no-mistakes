use super::runner::{is_managed_runner, is_managed_runner_only, parse_runs_on_values};
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use globset::GlobSet;
use std::path::Path;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/no-git-identity-mutation/fixture")
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
fn read_only_shell_forms_not_flagged() {
    let findings = run_on_source(
        "git config user.name\n\
         git config --get user.email || echo missing\n\
         git config user.email 2>/dev/null\n",
    );
    assert!(findings.is_empty());
}

#[test]
fn array_form_flagged() {
    let findings = run_on_source("exec('git', 'config', 'user.name', 'Bot')\n");
    assert_eq!(findings.len(), 1);
}

#[test]
fn read_only_array_form_not_flagged() {
    let findings = run_on_source(
        "exec('git', 'config', 'user.name')\n\
         exec('git', 'config', '--get', 'user.email', 'fallback')\n",
    );
    assert!(findings.is_empty());
}

#[test]
fn helper_form_flagged() {
    let findings = run_on_source("git('config', 'user.name', 'Bot')\n");
    assert_eq!(findings.len(), 1);
}

#[test]
fn read_only_helper_form_not_flagged() {
    let findings = run_on_source(
        "git('config', 'user.email')\n\
         git('config', '--get', 'user.email', 'fallback')\n",
    );
    assert!(findings.is_empty());
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
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join(".github/workflows")).unwrap();
    let content = "runs-on: 'ubuntu-latest'\ngit config user.name \"Bot\"\n";
    let path = tmp.path().join(".github/workflows/ci.yml");
    std::fs::write(&path, content).unwrap();
    let mut cond_builder = globset::GlobSetBuilder::new();
    cond_builder.add(globset::Glob::new(".github/workflows/*.yml").unwrap());
    let cond_set = cond_builder.build().unwrap();
    let findings = check_file(&path, tmp.path(), &empty_globset(), &cond_set, &patterns());
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
    let tmp = tempfile::tempdir().unwrap();
    let script = tmp.path().join("scripts").join("setup.sh");
    std::fs::create_dir_all(script.parent().unwrap()).unwrap();
    std::fs::write(&script, "git config user.name Bot\n").unwrap();
    let config = config_with_options("excludePaths: [\"scripts/**\"]");
    let findings = check_with_files(tmp.path(), &config, &[script]).unwrap();
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

#[test]
fn non_script_file_not_flagged() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("README.md");
    std::fs::write(&path, "git config user.name \"Bot\"\n").unwrap();
    let findings = check_file(
        &path,
        tmp.path(),
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
    let findings = run_on_source("# git config user.name \"Bot\"\n");
    assert!(
        findings.is_empty(),
        "commented git config should not be flagged"
    );
}

#[test]
fn echo_line_not_flagged() {
    let findings = run_on_source("echo \"git config user.name Bot\"\n");
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
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("deploy");
    std::fs::write(&path, "#!/bin/bash\ngit config user.name \"Bot\"\n").unwrap();
    let findings = check_file(
        &path,
        tmp.path(),
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
    let tmp = tempfile::tempdir().unwrap();
    let script = tmp.path().join("scripts").join("setup.sh");
    std::fs::create_dir_all(script.parent().unwrap()).unwrap();
    std::fs::write(&script, "git config user.name Bot\n").unwrap();
    // Config with repository scope — check() should apply target_roots
    let config = config_with_options("{}");
    let files = vec![script];
    let findings = check_with_files(tmp.path(), &config, &files).unwrap();
    assert!(!findings.is_empty(), "should find violation in script");
}

#[test]
fn is_managed_runner_only_inline_list_all_managed() {
    let content = "runs-on: [ubuntu-latest, windows-latest]\n";
    assert!(
        is_managed_runner_only(content),
        "bracket list of managed runners should be recognized"
    );
}

#[test]
fn is_managed_runner_only_inline_list_mixed() {
    let content = "runs-on: [ubuntu-latest, self-hosted]\n";
    assert!(
        !is_managed_runner_only(content),
        "bracket list with self-hosted should not be recognized as managed-only"
    );
}

#[test]
fn is_managed_runner_only_trailing_comma_skips_empty_element() {
    // Exercises line 95: `continue` when a split segment is empty (e.g. trailing comma).
    let content = "runs-on: [ubuntu-latest,]\n";
    assert!(
        is_managed_runner_only(content),
        "trailing comma produces empty element that should be skipped, not treated as unmanaged"
    );
}

#[test]
fn is_managed_runner_only_multiline_list_managed() {
    // Exercises EW60S: multi-line YAML list form strips leading "- " before matching.
    let content = "runs-on:\n  - ubuntu-latest\n";
    assert!(
        is_managed_runner_only(content),
        "multi-line YAML list with managed runner should be recognized"
    );
}

#[test]
fn is_managed_runner_only_multiline_list_self_hosted() {
    let content = "runs-on:\n  - self-hosted\n";
    assert!(
        !is_managed_runner_only(content),
        "multi-line YAML list with self-hosted runner should not be recognized as managed-only"
    );
}

#[test]
fn is_managed_runner_only_multiline_list_later_self_hosted() {
    let content = "runs-on:\n  - ubuntu-latest\n  - self-hosted\n";
    assert!(
        !is_managed_runner_only(content),
        "multi-line YAML list must inspect every runner entry"
    );
}

#[test]
fn is_managed_runner_only_multiline_list_skips_blank_and_comment_lines() {
    let content = "runs-on:\n\n  # hosted runners\n  - ubuntu-latest\n";
    assert!(
        is_managed_runner_only(content),
        "blank and comment lines inside runs-on lists should be ignored"
    );
}

#[test]
fn is_managed_runner_only_mapping_labels_managed() {
    let content = "runs-on:\n  group: hosted\n  labels: ubuntu-latest\n";
    assert!(
        is_managed_runner_only(content),
        "mapping labels with a managed runner should be recognized"
    );
}

#[test]
fn is_managed_runner_only_mapping_labels_self_hosted() {
    let content = "runs-on:\n  group: hosted\n  labels: self-hosted\n";
    assert!(
        !is_managed_runner_only(content),
        "mapping labels with self-hosted should not be recognized as managed-only"
    );
}

#[test]
fn is_managed_runner_only_multiline_non_list_stops_collection() {
    let content = "runs-on:\n  name: ubuntu-runners\n  - ubuntu-latest\n";
    assert!(
        !is_managed_runner_only(content),
        "non-list YAML values should not be treated as managed runner entries"
    );
}

#[test]
fn line_continued_git_config_is_flagged() {
    // Exercises EW60T: shell regex now allows \<newline> line continuations.
    let findings = run_on_source("git config \\\n  user.name \"Bot\"\n");
    assert_eq!(
        findings.len(),
        1,
        "line-continued git config user.name should be flagged"
    );
}
