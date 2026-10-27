use super::candidates::{
    collect_shell_files, collect_shell_files_with_sources, filtered_shell_files, has_bash_shebang,
};
use super::execution::{handle_shellcheck_result, parse_affected_files};
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn make_exit_status(code: i32) -> std::process::ExitStatus {
    #[cfg(unix)]
    {
        std::os::unix::process::ExitStatusExt::from_raw(code << 8)
    }
    #[cfg(not(unix))]
    {
        std::process::Command::new("cmd")
            .args(["/C", &format!("exit {}", code)])
            .status()
            .unwrap()
    }
}

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
            .join("../../test-cases/rules/shellcheck-runner/fixture")
            .join(subpath),
    )
}

fn source_candidates_fixture_root() -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/shellcheck-runner/source-candidates"),
    )
}

fn shellcheck_available() -> bool {
    std::process::Command::new("shellcheck")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

mod tracked_only;

#[test]
fn pass_fixture_is_clean_or_reports_missing_shellcheck() {
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
    } else {
        assert!(!findings.is_empty());
        assert!(findings
            .iter()
            .all(|finding| finding.target.as_deref() == Some("unanalyzable")));
    }
}

#[test]
fn fail_fixture_reports_issues_or_missing_shellcheck() {
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
    assert!(!findings.is_empty());
}

#[test]
fn no_shell_files_returns_empty() {
    let root = fixture_root("no-shell-files");
    let config = config_with_rule("{shellcheck: {severity: warning}}");
    let findings = check(&root, &config).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn collect_shell_files_finds_sh_files() {
    let root = fixture_root("collect-sh");
    let sh = root.join("setup.sh");
    let md = root.join("readme.md");
    let opts = Options::default();
    let files = vec![sh.clone(), md];
    let candidates = collect_shell_files(&root, &opts, &files, &[]);
    assert_eq!(candidates, vec![sh]);
}

#[test]
fn collect_shell_files_includes_explicit_shell_files() {
    let root = fixture_root("explicit-deploy");
    let script = root.join("deploy");
    let opts = Options {
        shell_files: vec!["deploy".to_string()],
        ..Default::default()
    };
    let files = vec![];
    let candidates = collect_shell_files(&root, &opts, &files, &[]);
    assert_eq!(candidates, vec![script]);
}

#[test]
fn collect_shell_files_detects_shebang_in_dir() {
    let root = fixture_root("shebang-scripts");
    let script = root.join("scripts/deploy");
    let opts = Options {
        shebang_dirs: vec!["scripts".to_string()],
        ..Default::default()
    };
    let files = vec![script.clone()];
    let candidates = collect_shell_files(&root, &opts, &files, &[]);
    assert_eq!(candidates, vec![script]);
}

#[test]
fn collect_shell_files_with_sources_covers_configured_candidates() {
    let root = source_candidates_fixture_root();
    let bash_script = root.join("scripts/deploy");
    let non_shell_script = root.join("scripts/generate");
    let extension_script = root.join("scripts/setup.sh");
    let explicit_script = root.join("explicit");
    let outside_script = root.join("outside/deploy");
    let files = vec![
        bash_script.clone(),
        non_shell_script,
        extension_script.clone(),
        outside_script,
    ];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let opts = Options {
        shell_files: vec!["explicit".to_string()],
        shebang_dirs: vec!["scripts".to_string()],
        ..Default::default()
    };

    let candidates = collect_shell_files_with_sources(&root, &opts, &files, &[], sources.as_ref());

    assert_eq!(
        candidates,
        vec![explicit_script, bash_script, extension_script]
    );
}

#[test]
fn collect_shell_files_with_sources_handles_root_and_target_scope() {
    let root = source_candidates_fixture_root();
    let root_script = root.join("root-script");
    let explicit_script = root.join("explicit");
    let files = vec![root_script.clone()];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let opts = Options {
        shell_files: vec!["explicit".to_string(), "missing".to_string()],
        shebang_dirs: vec![String::new()],
        ..Default::default()
    };

    let candidates = collect_shell_files_with_sources(
        &root,
        &opts,
        &files,
        &[root.join("scripts")],
        sources.as_ref(),
    );

    assert_eq!(candidates, vec![root_script]);
    assert!(!candidates.contains(&explicit_script));
}

#[test]
fn has_bash_shebang_detects_bash() {
    let path = fixture_root("bash-shebang").join("deploy");
    assert!(has_bash_shebang(&path));
}

#[test]
fn has_bash_shebang_false_for_non_shell() {
    let path = fixture_root("python-shebang").join("script.py");
    assert!(!has_bash_shebang(&path));
}

#[test]
fn check_with_files_works() {
    let root = fixture_root("check-with-files");
    let sh = root.join("script.sh");
    let config = config_with_rule("{shellcheck: {severity: warning}}");
    // Should not error — may or may not have findings depending on shellcheck
    let result = check_with_files(&root, &config, &[sh]);
    assert!(result.is_ok());
}

#[test]
fn shebang_dir_empty_string_uses_root() {
    // Exercises line 90: empty dir_rel → root.to_path_buf().
    let root = fixture_root("shebang-root");
    let script = root.join("deploy");
    let opts = Options {
        shebang_dirs: vec![String::new()], // empty string → root itself
        ..Default::default()
    };
    let files = vec![script.clone()];
    let candidates = collect_shell_files(&root, &opts, &files, &[]);
    assert!(
        candidates.contains(&script),
        "root shebang script should be included"
    );
}

#[test]
fn shebang_dir_skips_file_in_wrong_parent() {
    // File not under the shebang_dir → starts_with fails → skip.
    let root = fixture_root("shebang-wrong-parent");
    let wrong_script = root.join("other/deploy");
    let opts = Options {
        shebang_dirs: vec!["scripts".to_string()],
        ..Default::default()
    };
    let files = vec![wrong_script];
    let candidates = collect_shell_files(&root, &opts, &files, &[]);
    assert!(
        candidates.is_empty(),
        "file in wrong parent should be skipped"
    );
}

#[test]
fn shebang_dir_skips_sh_files_already_collected() {
    // Exercises line 103: .sh extension → skip in shebang_dirs loop.
    let root = fixture_root("shebang-sh-dedup");
    let sh_file = root.join("scripts/setup.sh");
    let opts = Options {
        shebang_dirs: vec!["scripts".to_string()],
        ..Default::default()
    };
    let files = vec![sh_file.clone()];
    let candidates = collect_shell_files(&root, &opts, &files, &[]);
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
    let root = fixture_root("echo-hi");
    let sh = root.join("test.sh");
    let opts = Options {
        shellcheck: ShellcheckOptions {
            severity: String::new(), // empty → DEFAULT_SEVERITY
        },
        ..Default::default()
    };
    // Missing ShellCheck is a structured incomplete result, not an invocation error.
    let result = run_shellcheck(&root, &opts, &[sh]);
    assert!(result.is_ok());
}

#[test]
fn shebang_dir_file_with_no_parent_is_skipped() {
    // Path::new("/") is not under the shebang_dir → starts_with fails → skip.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let opts = Options {
        shebang_dirs: vec!["scripts".to_string()],
        ..Default::default()
    };
    let files = vec![std::path::PathBuf::from("/")];
    let candidates = collect_shell_files(root, &opts, &files, &[]);
    assert!(
        candidates.is_empty(),
        "file not under shebang dir should be skipped"
    );
}

#[test]
fn shebang_dir_includes_nested_script() {
    // Exercises recursive shebang dir scanning: files in subdirs should be included.
    let root = fixture_root("nested-shebang");
    let nested = root.join("scripts/tools/deploy");
    let opts = Options {
        shebang_dirs: vec!["scripts".to_string()],
        ..Default::default()
    };
    let files = vec![nested.clone()];
    let candidates = collect_shell_files(&root, &opts, &files, &[]);
    assert_eq!(
        candidates,
        vec![nested],
        "nested shebang script should be included"
    );
}

#[test]
fn collect_shell_files_respects_target_roots() {
    // Exercises EW60V: explicit shell_files outside target_roots should be skipped.
    let root = fixture_root("target-roots");
    let proj_dir = root.join("proj");
    let inside = proj_dir.join("run.sh");
    let opts = Options {
        shell_files: vec!["proj/run.sh".to_string(), "other/deploy".to_string()],
        ..Default::default()
    };
    let target_roots = vec![proj_dir.clone()];
    let candidates = collect_shell_files(&root, &opts, &[], &target_roots);
    assert_eq!(
        candidates,
        vec![inside],
        "shell_files outside target_roots should be excluded"
    );
}

#[test]
fn explicit_shell_files_respect_rule_path_filters() {
    let root = fixture_root("path-filters");
    let included = root.join("scripts/run.sh");
    let opts = Options {
        shell_files: vec![
            "scripts/run.sh".to_string(),
            "scripts/generated.sh".to_string(),
        ],
        ..Default::default()
    };
    let mut config = config_with_rule("{shellcheck: {severity: warning}}");
    config.rules[0].exclude = vec!["scripts/generated.sh".to_string()];
    let rule_filter =
        super::super::path_filter::RulePathFilter::new(&root, &config, &config.rules[0]).unwrap();

    let candidates = filtered_shell_files(&root, &opts, &[], &[], &rule_filter);

    assert_eq!(candidates, vec![included]);
}

#[test]
fn parse_affected_files_empty_stdout_returns_empty() {
    let files: Vec<std::path::PathBuf> = vec![];
    assert!(parse_affected_files("", &files).is_empty());
}

#[test]
fn parse_affected_files_line_without_colon_skipped() {
    let sh = fixture_root("parse-affected").join("a.sh");
    let stdout = "no colon here\n";
    let result = parse_affected_files(stdout, std::slice::from_ref(&sh));
    assert!(result.is_empty());
}

#[test]
fn parse_affected_files_unknown_file_skipped() {
    let sh = fixture_root("parse-affected").join("a.sh");
    let stdout = format!(
        "{}:1:1: warning: blah [SC2086]\n/tmp/unknown.sh:1:1: warning: blah [SC2086]\n",
        sh.display()
    );
    let result = parse_affected_files(&stdout, std::slice::from_ref(&sh));
    assert_eq!(result, vec![sh]);
}

#[test]
fn parse_affected_files_deduplicates() {
    let sh = fixture_root("parse-affected").join("a.sh");
    let stdout = format!(
        "{}:1:1: warning: blah [SC2086]\n{}:2:1: warning: blah2 [SC2087]\n",
        sh.display(),
        sh.display()
    );
    let result = parse_affected_files(&stdout, std::slice::from_ref(&sh));
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], sh);
}

#[test]
fn run_shellcheck_rejects_invalid_severity() {
    let root = fixture_root("echo-hi");
    let sh = root.join("test.sh");
    let opts = Options {
        shellcheck: ShellcheckOptions {
            severity: "invalid_value".to_string(),
        },
        ..Default::default()
    };
    let result = run_shellcheck(&root, &opts, &[sh]);
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("invalid shellcheck severity"));
    assert!(
        error.contains(r#"invalid shellcheck severity: "invalid_value"."#)
            && error.contains("error, warning, info, style"),
        "{}",
        error
    );
}

#[path = "tests/incomplete.rs"]
mod incomplete;
