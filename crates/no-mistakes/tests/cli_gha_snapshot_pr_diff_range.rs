use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-pr-diff-range";

#[test]
fn github_actions_pr_diff_range_json_matches_snapshot() {
    let cases: Vec<_> = [
        "report",
        "pass",
        "suppression",
        "env-and-heredoc",
        "event-fallbacks",
        "shell-guard-elif",
        "shell-guard-preamble",
        "shell-guard-runner-event",
        "shell-guard-chain",
        "shell-guard-regex",
        "shell-guard-separator",
        "shell-guard-command-expansion",
        "shell-guard-dynamic-target",
        "shell-guard-condition-mutation",
        "shell-guard-time-control",
    ]
    .into_iter()
    .map(|name| {
        (
            name,
            rules_path(RULE, "fixture", name),
            rules_path(RULE, "configs", "report.yml"),
        )
    })
    .collect();
    assert_saved_fixture("github_actions_pr_diff_range", &cases);
}

#[test]
fn github_actions_pr_diff_range_consistent_pairs_json_matches_snapshot() {
    let cases: Vec<_> = [
        ("advise", "consistent-pairs.yml"),
        ("report", "consistent-pairs-report.yml"),
        ("ignore", "consistent-pairs-ignore.yml"),
    ]
    .into_iter()
    .map(|(label, config)| {
        (
            label,
            rules_path(RULE, "fixture", "consistent-pairs"),
            rules_path(RULE, "configs", config),
        )
    })
    .collect();
    assert_saved_fixture("github_actions_pr_diff_range_consistent_pairs", &cases);

    let root = rules_path(RULE, "fixture", "consistent-pairs");
    let invalid = rules_path(RULE, "configs", "consistent-pairs-invalid.yml");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .current_dir(&root)
        .args(["--lock-timeout", "0", "check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(&invalid)
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("revisionPairs[0]"), "{stderr}");
}
