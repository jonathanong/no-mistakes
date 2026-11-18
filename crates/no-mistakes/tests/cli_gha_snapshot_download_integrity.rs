use super::check_json_snapshot::{assert_saved_fixture, rules_path};

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
            "github_actions_download_integrity_shell_source",
            "shell-source",
            &["fail", "shell-source", "shell-source-low"][..],
        ),
        (
            "github_actions_download_integrity_strict_sequencing",
            "strict-sequencing",
            &["fail", "strict-sequencing", "strict-sequencing-low"][..],
        ),
    ] {
        let cases = configs
            .iter()
            .map(|&config| {
                (
                    config,
                    rules_path(RULE, "fixture", fixture),
                    rules_path(RULE, "configs", &format!("{config}.yml")),
                )
            })
            .collect::<Vec<_>>();
        assert_saved_fixture(snapshot, &cases);
    }
}
