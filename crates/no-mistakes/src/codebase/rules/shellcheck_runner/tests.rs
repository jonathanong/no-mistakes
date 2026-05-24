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
            .join("../../fixtures/rules/shellcheck-runner")
            .join(subpath),
    )
}

fn shellcheck_available() -> bool {
    std::process::Command::new("shellcheck")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

#[test]
fn pass_fixture_has_no_findings_or_skips_without_shellcheck() {
    let root = fixture_root("pass");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    if shellcheck_available() {
        assert!(
            findings.is_empty(),
            "expected no findings, got: {findings:?}"
        );
    }
    // Without shellcheck installed the rule silently returns no findings too
}

#[test]
fn fail_fixture_has_findings_or_skips_without_shellcheck() {
    let root = fixture_root("fail");
    let config_path = root.join(".no-mistakes.yml");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap(),
    )
    .unwrap();
    if shellcheck_available() {
        assert!(
            !findings.is_empty(),
            "expected findings for bad shell script"
        );
    }
    // Without shellcheck the rule is a no-op; that's acceptable
}

#[test]
fn no_shell_files_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::fs::write(root.join("readme.md"), "# Hello\n").unwrap();
    let config = config_with_rule("{shellcheck: {severity: warning}}");
    let findings = check(root, &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn collect_shell_files_finds_sh_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let sh = root.join("setup.sh");
    std::fs::write(&sh, "#!/bin/bash\necho hi\n").unwrap();
    let md = root.join("readme.md");
    std::fs::write(&md, "# Title\n").unwrap();
    let opts = Options::default();
    let files = vec![sh.clone(), md];
    let candidates = collect_shell_files(root, &opts, &files);
    assert_eq!(candidates, vec![sh]);
}

#[test]
fn collect_shell_files_includes_explicit_shell_files() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let script = root.join("deploy");
    std::fs::write(&script, "#!/bin/bash\necho deploy\n").unwrap();
    let opts = Options {
        shell_files: vec!["deploy".to_string()],
        ..Default::default()
    };
    let files = vec![];
    let candidates = collect_shell_files(root, &opts, &files);
    assert_eq!(candidates, vec![script]);
}

#[test]
fn collect_shell_files_detects_shebang_in_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let scripts_dir = root.join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();
    let script = scripts_dir.join("deploy");
    std::fs::write(&script, "#!/bin/bash\necho deploy\n").unwrap();
    let opts = Options {
        shebang_dirs: vec!["scripts".to_string()],
        ..Default::default()
    };
    let files = vec![script.clone()];
    let candidates = collect_shell_files(root, &opts, &files);
    assert_eq!(candidates, vec![script]);
}

#[test]
fn has_bash_shebang_detects_bash() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("deploy");
    std::fs::write(&path, "#!/bin/bash\necho hi\n").unwrap();
    assert!(has_bash_shebang(&path));
}

#[test]
fn has_bash_shebang_false_for_non_shell() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("script.py");
    std::fs::write(&path, "#!/usr/bin/env python3\nprint('hi')\n").unwrap();
    assert!(!has_bash_shebang(&path));
}

#[test]
fn check_with_files_works() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let sh = root.join("script.sh");
    std::fs::write(&sh, "#!/bin/bash\nset -euo pipefail\necho hi\n").unwrap();
    let config = config_with_rule("{shellcheck: {severity: warning}}");
    // Should not error — may or may not have findings depending on shellcheck
    let result = check_with_files(root, &config, &[sh]);
    assert!(result.is_ok());
}

#[test]
fn shebang_dir_empty_string_uses_root() {
    // Exercises line 90: empty dir_rel → root.to_path_buf().
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let script = root.join("deploy");
    std::fs::write(&script, "#!/bin/bash\necho deploy\n").unwrap();
    let opts = Options {
        shebang_dirs: vec![String::new()], // empty string → root itself
        ..Default::default()
    };
    let files = vec![script.clone()];
    let candidates = collect_shell_files(root, &opts, &files);
    assert!(
        candidates.contains(&script),
        "root shebang script should be included"
    );
}

#[test]
fn shebang_dir_skips_file_in_wrong_parent() {
    // Exercises line 99: parent != dir → skip.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let scripts_dir = root.join("scripts");
    let other_dir = root.join("other");
    std::fs::create_dir_all(&scripts_dir).unwrap();
    std::fs::create_dir_all(&other_dir).unwrap();
    let wrong_script = other_dir.join("deploy");
    std::fs::write(&wrong_script, "#!/bin/bash\necho deploy\n").unwrap();
    let opts = Options {
        shebang_dirs: vec!["scripts".to_string()],
        ..Default::default()
    };
    let files = vec![wrong_script];
    let candidates = collect_shell_files(root, &opts, &files);
    assert!(
        candidates.is_empty(),
        "file in wrong parent should be skipped"
    );
}

#[test]
fn shebang_dir_skips_sh_files_already_collected() {
    // Exercises line 103: .sh extension → skip in shebang_dirs loop.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let scripts_dir = root.join("scripts");
    std::fs::create_dir_all(&scripts_dir).unwrap();
    let sh_file = scripts_dir.join("setup.sh");
    std::fs::write(&sh_file, "#!/bin/bash\necho hi\n").unwrap();
    let opts = Options {
        shebang_dirs: vec!["scripts".to_string()],
        ..Default::default()
    };
    let files = vec![sh_file.clone()];
    let candidates = collect_shell_files(root, &opts, &files);
    // setup.sh was already added via the .sh extension pass; it should appear
    // exactly once (dedup ensures no duplicate).
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0], sh_file);
}

#[test]
fn has_bash_shebang_returns_false_for_nonexistent_file() {
    // Exercises line 127: File::open fails → return false.
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("nonexistent");
    assert!(!has_bash_shebang(&path));
}

#[test]
fn run_shellcheck_uses_default_severity_when_empty() {
    // Exercises line 145: opts.shellcheck.severity.is_empty() → DEFAULT_SEVERITY.
    let tmp = tempfile::tempdir().unwrap();
    let sh = tmp.path().join("test.sh");
    std::fs::write(&sh, "#!/bin/bash\necho hi\n").unwrap();
    let opts = Options {
        shellcheck: ShellcheckOptions {
            severity: String::new(), // empty → DEFAULT_SEVERITY
        },
        ..Default::default()
    };
    // run_shellcheck should not error — may return Ok(empty) if shellcheck not installed
    let result = run_shellcheck(tmp.path(), &opts, &[sh]);
    assert!(result.is_ok());
}

#[test]
fn shebang_dir_file_with_no_parent_is_skipped() {
    // Exercises line 96: path.parent() returns None for the filesystem root path,
    // so the file is skipped in the shebang_dirs loop.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let opts = Options {
        shebang_dirs: vec!["scripts".to_string()],
        ..Default::default()
    };
    // Path::new("/") has no parent — parent() returns None
    let files = vec![std::path::PathBuf::from("/")];
    let candidates = collect_shell_files(root, &opts, &files);
    assert!(
        candidates.is_empty(),
        "file with no parent should be skipped in shebang dir loop"
    );
}

#[test]
fn handle_shellcheck_result_not_found_returns_empty() {
    // Exercises the NotFound arm: shellcheck binary not installed → silent Ok(empty).
    let tmp = tempfile::tempdir().unwrap();
    let err = std::io::Error::new(std::io::ErrorKind::NotFound, "shellcheck not found");
    let result = handle_shellcheck_result(tmp.path(), &[], Err(err));
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn handle_shellcheck_result_other_error_propagates() {
    // Exercises the generic Err arm: any non-NotFound I/O error → Err.
    let tmp = tempfile::tempdir().unwrap();
    let err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
    let result = handle_shellcheck_result(tmp.path(), &[], Err(err));
    assert!(result.is_err());
}

#[test]
fn handle_shellcheck_result_success_returns_empty() {
    // Exercises the Ok(output) arm with exit code 0 → no findings.
    let tmp = tempfile::tempdir().unwrap();
    let output = std::process::Output {
        status: std::os::unix::process::ExitStatusExt::from_raw(0),
        stdout: Vec::new(),
        stderr: Vec::new(),
    };
    let result = handle_shellcheck_result(tmp.path(), &[], Ok(output));
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn handle_shellcheck_result_reports_only_affected_files() {
    // Only files mentioned in gcc-format stdout get findings; clean files are skipped.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let sh1 = root.join("a.sh");
    let sh2 = root.join("b.sh");
    std::fs::write(&sh1, "#!/bin/bash\nfoo\n").unwrap();
    std::fs::write(&sh2, "#!/bin/bash\nbar\n").unwrap();
    let gcc_line = format!("{}:2:1: warning: blah [SC2006]\n", sh1.display());
    let output = std::process::Output {
        status: std::os::unix::process::ExitStatusExt::from_raw(1 << 8),
        stdout: gcc_line.into_bytes(),
        stderr: Vec::new(),
    };
    let result = handle_shellcheck_result(root, &[sh1.clone(), sh2], Ok(output));
    let findings = result.unwrap();
    assert_eq!(
        findings.len(),
        1,
        "only the flagged file should get a finding"
    );
    assert!(findings[0].file.contains("a.sh"));
}

#[test]
fn parse_affected_files_empty_stdout_returns_empty() {
    let files: Vec<std::path::PathBuf> = vec![];
    assert!(parse_affected_files("", &files).is_empty());
}

#[test]
fn parse_affected_files_line_without_colon_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let sh = tmp.path().join("a.sh");
    std::fs::write(&sh, "#!/bin/bash\n").unwrap();
    let stdout = "no colon here\n";
    let result = parse_affected_files(stdout, std::slice::from_ref(&sh));
    assert!(result.is_empty());
}

#[test]
fn parse_affected_files_unknown_file_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let sh = tmp.path().join("known.sh");
    std::fs::write(&sh, "#!/bin/bash\n").unwrap();
    let stdout = format!(
        "{}:1:1: warning: blah [SC2086]\n/tmp/unknown.sh:1:1: warning: blah [SC2086]\n",
        sh.display()
    );
    let result = parse_affected_files(&stdout, std::slice::from_ref(&sh));
    assert_eq!(result, vec![sh]);
}

#[test]
fn parse_affected_files_deduplicates() {
    let tmp = tempfile::tempdir().unwrap();
    let sh = tmp.path().join("a.sh");
    std::fs::write(&sh, "#!/bin/bash\n").unwrap();
    let stdout = format!(
        "{}:1:1: warning: blah [SC2086]\n{}:2:1: warning: blah2 [SC2087]\n",
        sh.display(),
        sh.display()
    );
    let result = parse_affected_files(&stdout, std::slice::from_ref(&sh));
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], sh);
}
