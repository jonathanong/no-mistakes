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
