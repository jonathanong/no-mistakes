use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "github-actions-run-command-policy";

#[test]
fn github_actions_run_command_policy_json_matches_snapshot() {
    let policy = rules_path(RULE, "configs", "policy.yml");
    let policy_shell = rules_path(RULE, "configs", "policy-shell.yml");
    let empty = rules_path(RULE, "configs", "empty.yml");
    let report = rules_path(RULE, "configs", "unanalyzable-report.yml");
    let ignore = rules_path(RULE, "configs", "unanalyzable-ignore.yml");
    assert_saved_fixture(
        "github_actions_run_command_policy",
        &[
            (
                "reported",
                rules_path(RULE, "fixture", "reported"),
                policy.clone(),
            ),
            (
                "disabled-file",
                rules_path(RULE, "fixture", "suppressions"),
                policy.clone(),
            ),
            (
                "compliant",
                rules_path(RULE, "fixture", "compliant"),
                policy.clone(),
            ),
            (
                "unparsable",
                rules_path(RULE, "fixture", "unparsable"),
                policy,
            ),
            (
                "shell-scripts",
                rules_path(RULE, "fixture", "shell-scripts"),
                policy_shell,
            ),
            (
                "empty-opt-in",
                rules_path(RULE, "fixture", "reported"),
                empty,
            ),
            (
                "unparsable-report",
                rules_path(RULE, "fixture", "unparsable"),
                report,
            ),
            (
                "unparsable-ignore",
                rules_path(RULE, "fixture", "unparsable"),
                ignore,
            ),
        ],
    );
}

#[test]
fn github_actions_run_command_policy_invalid_options_fail_early_with_exit_2() {
    let root = rules_path(RULE, "fixture", "shell-scripts");
    let config = rules_path(RULE, "configs", "invalid-shell-glob.yml");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["--lock-timeout", "0", "--jobs", "1", "check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(&config)
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty(), "stdout must be empty on exit 2");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("options.shellFiles"), "{stderr}");
}

#[test]
fn github_actions_run_command_policy_analysis_boundaries_json_matches_snapshot() {
    let root = rules_path(RULE, "fixture", "analysis-boundaries");
    assert_saved_fixture(
        "github_actions_run_command_policy_analysis_boundaries",
        &analysis_boundary_cases(&root),
    );
}

#[cfg(unix)]
#[test]
fn github_actions_run_command_policy_unreadable_json_matches_snapshot() {
    let fixture =
        super::saved_fixture::materialize_path(&rules_path(RULE, "fixture", "source-failure"));
    let root = fixture.path();
    let workflow = ".github/workflows/unavailable.yml";
    for args in [
        &["init", "--quiet"][..],
        &["add", "--force", "--", ".github/workflows"],
    ] {
        let output = std::process::Command::new("git")
            .current_dir(root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    // The index still describes a regular file after the worktree replacement.
    std::fs::rename(root.join("unavailable-workflow.link"), root.join(workflow)).unwrap();

    assert_saved_fixture(
        "github_actions_run_command_policy_unreadable",
        &analysis_boundary_cases(root),
    );
}

fn analysis_boundary_cases(root: &Path) -> [(&'static str, PathBuf, PathBuf); 3] {
    ["advise", "report", "ignore"].map(|mode| {
        (
            mode,
            root.to_path_buf(),
            rules_path(RULE, "configs", &format!("analysis-boundaries-{mode}.yml")),
        )
    })
}
