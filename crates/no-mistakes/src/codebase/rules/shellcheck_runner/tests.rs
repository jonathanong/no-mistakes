use super::candidates::{
    collect_shell_files, collect_shell_files_with_sources, filtered_shell_files, has_bash_shebang,
};
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

fn tracked_fixture() -> tempfile::TempDir {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/shellcheck-runner/tracked-only");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    std::fs::rename(
        fixture.path().join(".gitignore.fixture"),
        fixture.path().join(".gitignore"),
    )
    .unwrap();
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_force(
        fixture.path(),
        &[
            ".gitignore",
            "scripts/nested/tracked.sh",
            "scripts/bash",
            "scripts/posix",
            "scripts/modified.sh",
            "scripts/staged.sh",
            "scripts/unsupported",
            "scripts/explicit",
        ],
    );
    std::fs::copy(
        fixture.path().join("modified-after-stage.fixture"),
        fixture.path().join("scripts/modified.sh"),
    )
    .unwrap();
    fixture
}

fn candidate_names(paths: &[PathBuf], root: &Path) -> Vec<String> {
    let mut names = paths
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(root, path))
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn tracked_options() -> Options {
    Options {
        tracked_only: true,
        shebang_dirs: vec!["scripts".to_string()],
        shell_files: vec![
            "scripts/explicit".to_string(),
            "scripts/untracked-explicit".to_string(),
            "scripts/ignored-explicit".to_string(),
        ],
        ..Default::default()
    }
}

#[test]
fn unconfigured_rule_has_no_findings() {
    let fixture = tracked_fixture();
    assert!(check(
        fixture.path(),
        &crate::config::v2::NoMistakesConfig::default()
    )
    .unwrap()
    .is_empty());
}

#[test]
fn tracked_only_rejects_non_boolean_options_in_both_paths() {
    let root = fixture_root("pass");
    let config = config_with_rule("{trackedOnly: not-a-bool}");
    let files = Vec::new();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &files);
    let sources = crate::codebase::rules::source_store_for_files(&files);

    assert!(check(&root, &config).is_err());
    assert!(
        check_with_files_sources_and_snapshot(&root, &config, &files, &sources, &snapshot).is_err()
    );
}

#[test]
fn prepared_scan_with_no_candidates_has_no_findings() {
    let root = fixture_root("pass");
    let config = config_with_rule("{}");
    let rule_filter =
        super::super::path_filter::RulePathFilter::new(&root, &config, &config.rules[0]).unwrap();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&[]);

    assert!(scan_with_sources(
        &root,
        &Options::default(),
        &[],
        &[],
        &rule_filter,
        &sources,
        &snapshot,
    )
    .unwrap()
    .is_empty());
}

#[test]
fn multiple_shellcheck_findings_are_sorted() {
    let fixture = tracked_fixture();
    let root = fixture.path();
    let nested = root.join("scripts/nested/tracked.sh");
    let staged = root.join("scripts/staged.sh");
    let output = std::process::Output {
        status: make_exit_status(1),
        stdout: format!(
            "{}:2:1: warning: first [SC2006]\n{}:2:1: warning: second [SC2006]\n",
            staged.display(),
            nested.display()
        )
        .into_bytes(),
        stderr: Vec::new(),
    };

    let findings = handle_shellcheck_result(root, &[staged, nested], Ok(output)).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        ["scripts/nested/tracked.sh", "scripts/staged.sh"]
    );
}

#[test]
fn tracked_only_selects_indexed_shell_candidates() {
    let fixture = tracked_fixture();
    let root = fixture.path();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let files = snapshot.paths_for(root);
    let config = config_with_rule("{trackedOnly: true}");
    let rule_filter =
        super::super::path_filter::RulePathFilter::new(root, &config, &config.rules[0]).unwrap();
    let options = tracked_options();
    let sources = snapshot.source_store_for(root);

    for selected in [
        select_shell_candidates(root, &options, &files, &[], &rule_filter, &snapshot, None),
        select_shell_candidates(
            root,
            &options,
            &files,
            &[],
            &rule_filter,
            &snapshot,
            Some(&sources),
        ),
    ] {
        assert_eq!(
            candidate_names(&selected, root),
            [
                "scripts/bash",
                "scripts/explicit",
                "scripts/modified.sh",
                "scripts/nested/tracked.sh",
                "scripts/posix",
                "scripts/staged.sh",
            ]
        );
    }
}

#[test]
fn default_selection_keeps_visible_untracked_and_explicit_files() {
    let fixture = tracked_fixture();
    let root = fixture.path();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let files = snapshot.paths_for(root);
    let config = config_with_rule("{}");
    let rule_filter =
        super::super::path_filter::RulePathFilter::new(root, &config, &config.rules[0]).unwrap();
    let mut options = tracked_options();
    options.tracked_only = false;

    let selected =
        select_shell_candidates(root, &options, &files, &[], &rule_filter, &snapshot, None);
    let names = candidate_names(&selected, root);

    assert!(names.contains(&"scripts/untracked.sh".to_string()));
    assert!(names.contains(&"scripts/untracked-explicit".to_string()));
    assert!(names.contains(&"scripts/ignored-explicit".to_string()));
    assert!(!names.contains(&"scripts/unsupported".to_string()));
    assert!(!names.contains(&"scripts/ignored.sh".to_string()));
}

#[test]
fn tracked_only_uses_ignore_aware_visible_fallback_outside_git() {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/shellcheck-runner/tracked-only");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path();
    std::fs::rename(root.join(".gitignore.fixture"), root.join(".gitignore")).unwrap();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let files = snapshot.paths_for(root);
    let config = config_with_rule("{trackedOnly: true}");
    let rule_filter =
        super::super::path_filter::RulePathFilter::new(root, &config, &config.rules[0]).unwrap();

    let selected = select_shell_candidates(
        root,
        &tracked_options(),
        &files,
        &[],
        &rule_filter,
        &snapshot,
        None,
    );
    let names = candidate_names(&selected, root);

    assert!(names.contains(&"scripts/untracked.sh".to_string()));
    assert!(names.contains(&"scripts/untracked-explicit".to_string()));
    assert!(!names.contains(&"scripts/ignored.sh".to_string()));
    assert!(!names.contains(&"scripts/ignored-explicit".to_string()));
}

#[test]
fn standalone_and_prepared_dispatch_agree_on_tracked_only_findings() {
    let fixture = tracked_fixture();
    let root = fixture.path();
    let config = config_with_rule("{trackedOnly: true, shebangDirs: [scripts]}");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let snapshot =
        crate::codebase::ts_source::VisiblePathSnapshot::new_observed(root, Some(observer.clone()));
    let files = snapshot.paths_for(root);
    let initial_discovery_roots = observer.snapshot().work["discovery.roots"];

    let standalone = check(root, &config).unwrap();
    let prepared =
        super::super::filesystem_dispatch::run_filesystem_rules_with_config_and_snapshot(
            root, &config, &files, &snapshot,
        )
        .unwrap();

    assert_eq!(
        observer.snapshot().work["discovery.roots"],
        initial_discovery_roots
    );
    let standalone_files = standalone
        .iter()
        .map(|finding| finding.file.as_str())
        .collect::<Vec<_>>();
    let prepared_files = prepared
        .iter()
        .map(|finding| finding.file.as_str())
        .collect::<Vec<_>>();
    assert_eq!(standalone_files, prepared_files);
    if shellcheck_available() {
        assert_eq!(prepared_files, ["scripts/modified.sh"]);
    }
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
    let candidates = collect_shell_files(root, &opts, &files, &[]);
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
    let candidates = collect_shell_files(root, &opts, &files, &[]);
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
    let candidates = collect_shell_files(root, &opts, &files, &[]);
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
    let candidates = collect_shell_files(root, &opts, &files, &[]);
    assert!(
        candidates.contains(&script),
        "root shebang script should be included"
    );
}

#[test]
fn shebang_dir_skips_file_in_wrong_parent() {
    // File not under the shebang_dir → starts_with fails → skip.
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
    let candidates = collect_shell_files(root, &opts, &files, &[]);
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
    let candidates = collect_shell_files(root, &opts, &files, &[]);
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
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let tools_dir = root.join("scripts/tools");
    std::fs::create_dir_all(&tools_dir).unwrap();
    let nested = tools_dir.join("deploy");
    std::fs::write(&nested, "#!/bin/bash\necho deploy\n").unwrap();
    let opts = Options {
        shebang_dirs: vec!["scripts".to_string()],
        ..Default::default()
    };
    let files = vec![nested.clone()];
    let candidates = collect_shell_files(root, &opts, &files, &[]);
    assert_eq!(
        candidates,
        vec![nested],
        "nested shebang script should be included"
    );
}

#[test]
fn collect_shell_files_respects_target_roots() {
    // Exercises EW60V: explicit shell_files outside target_roots should be skipped.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let proj_dir = root.join("proj");
    let other_dir = root.join("other");
    std::fs::create_dir_all(&proj_dir).unwrap();
    std::fs::create_dir_all(&other_dir).unwrap();
    let inside = proj_dir.join("run.sh");
    let outside = other_dir.join("deploy");
    std::fs::write(&inside, "#!/bin/bash\necho hi\n").unwrap();
    std::fs::write(&outside, "#!/bin/bash\necho deploy\n").unwrap();
    let opts = Options {
        shell_files: vec!["proj/run.sh".to_string(), "other/deploy".to_string()],
        ..Default::default()
    };
    let target_roots = vec![proj_dir.clone()];
    let candidates = collect_shell_files(root, &opts, &[], &target_roots);
    assert_eq!(
        candidates,
        vec![inside],
        "shell_files outside target_roots should be excluded"
    );
}

#[test]
fn explicit_shell_files_respect_rule_path_filters() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let included = root.join("scripts/run.sh");
    let excluded = root.join("scripts/generated.sh");
    std::fs::create_dir_all(root.join("scripts")).unwrap();
    std::fs::write(&included, "#!/bin/bash\necho hi\n").unwrap();
    std::fs::write(&excluded, "#!/bin/bash\necho generated\n").unwrap();
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
        super::super::path_filter::RulePathFilter::new(root, &config, &config.rules[0]).unwrap();

    let candidates = filtered_shell_files(root, &opts, &[], &[], &rule_filter);

    assert_eq!(candidates, vec![included]);
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
        status: make_exit_status(0),
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
        status: make_exit_status(1),
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

#[test]
fn run_shellcheck_rejects_invalid_severity() {
    let tmp = tempfile::tempdir().unwrap();
    let sh = tmp.path().join("test.sh");
    std::fs::write(&sh, "#!/bin/bash\necho hi\n").unwrap();
    let opts = Options {
        shellcheck: ShellcheckOptions {
            severity: "invalid_value".to_string(),
        },
        ..Default::default()
    };
    let result = run_shellcheck(tmp.path(), &opts, &[sh]);
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
