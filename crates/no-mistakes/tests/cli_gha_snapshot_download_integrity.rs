use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use super::git_tracked_rule::materialize_tracked;

const RULE: &str = "github-actions-download-integrity";

#[test]
fn github_actions_download_integrity_json_matches_snapshot() {
    assert_saved_fixture(
        "github_actions_download_integrity",
        &[
            (
                "fail",
                rules_path(RULE, "fixture", "fail"),
                rules_path(RULE, "configs", "fail.yml"),
            ),
            (
                "helpers",
                rules_path(RULE, "fixture", "helpers"),
                rules_path(RULE, "configs", "helpers.yml"),
            ),
            (
                "empty-hosts",
                rules_path(RULE, "fixture", "empty-hosts"),
                rules_path(RULE, "configs", "empty-hosts.yml"),
            ),
            (
                "pass",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "pass.yml"),
            ),
            (
                "abstain",
                rules_path(RULE, "fixture", "abstain"),
                rules_path(RULE, "configs", "abstain.yml"),
            ),
            (
                "edges",
                rules_path(RULE, "fixture", "edges"),
                rules_path(RULE, "configs", "edges.yml"),
            ),
            (
                "port-bracket",
                rules_path(RULE, "fixture", "fail"),
                rules_path(RULE, "configs", "port-bracket.yml"),
            ),
        ],
    );
}

#[test]
fn github_actions_download_integrity_scopes_and_sequencing_match_snapshot() {
    for (snapshot, fixture, configs) in [
        (
            "github_actions_download_integrity_step_env",
            "step-env",
            &["fail"][..],
        ),
        (
            "github_actions_download_integrity_step_env_unquoted",
            "step-env-unquoted",
            &["fail"][..],
        ),
        (
            "github_actions_download_integrity_builtin_writes",
            "builtin-writes",
            &["fail"][..],
        ),
        (
            "github_actions_download_integrity_shell_source",
            "shell-source",
            &["fail", "shell-source", "shell-source-low"][..],
        ),
        (
            "github_actions_download_integrity_coverage_controls",
            "coverage-controls",
            &["fail", "strict-sequencing", "strict-sequencing-low"][..],
        ),
        (
            "github_actions_download_integrity_empty_args",
            "empty-args",
            &["fail", "strict-sequencing"][..],
        ),
        (
            "github_actions_download_integrity_review_shapes",
            "review-shapes",
            &["fail", "strict-sequencing", "strict-sequencing-low"][..],
        ),
        (
            "github_actions_download_integrity_strict_sequencing",
            "strict-sequencing",
            &["fail", "strict-sequencing", "strict-sequencing-low"][..],
        ),
    ] {
        // Shell sources need a Git index; a `git archive` checkout has none.
        let tracked = (fixture == "shell-source")
            .then(|| materialize_tracked(&rules_path(RULE, "fixture", fixture)));
        let cases = configs
            .iter()
            .map(|&config| {
                (
                    config,
                    tracked.as_ref().map_or_else(
                        || rules_path(RULE, "fixture", fixture),
                        |dir| dir.path().to_path_buf(),
                    ),
                    rules_path(RULE, "configs", &format!("{config}.yml")),
                )
            })
            .collect::<Vec<_>>();
        assert_saved_fixture(snapshot, &cases);
    }
}
