use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::{Path, PathBuf};

const RULE: &str = "github-actions-run-command-policy";

#[test]
fn github_actions_run_command_policy_json_matches_snapshot() {
    let policy = rules_path(RULE, "configs", "policy.yml");
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
