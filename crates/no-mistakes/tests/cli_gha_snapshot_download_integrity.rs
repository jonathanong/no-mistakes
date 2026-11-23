use super::check_json_snapshot::{assert_saved_fixture, assert_saved_fixture_report, rules_path};
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

#[test]
fn github_actions_download_integrity_url_assignments_match_snapshot() {
    let cases = [
        ("max", "url-assignments.yml"),
        ("low", "url-assignments-low.yml"),
        ("strict", "url-assignments-strict.yml"),
    ]
    .map(|(label, config)| {
        (
            label,
            rules_path(RULE, "fixture", "url-assignments"),
            rules_path(RULE, "configs", config),
        )
    });
    let report = assert_saved_fixture_report(
        "github_actions_download_integrity_url_assignments",
        &cases,
        &[],
        &[],
    );
    let workflow = ".github/workflows/downloads.yml";
    for (label, lines) in [
        ("max", &[9, 19, 28, 29, 42][..]),
        (
            "low",
            &[9, 12, 19, 21, 22, 23, 24, 28, 29, 41, 42, 48, 49][..],
        ),
        ("strict", &[9, 19, 28, 29, 42, 51][..]),
    ] {
        let expected = lines
            .iter()
            .map(|&line| (workflow, line))
            .collect::<Vec<_>>();
        assert_finding_locations(&report[label], &expected);
    }
}

#[test]
fn github_actions_download_integrity_line_skips_match_snapshot() {
    let tracked = materialize_tracked(&rules_path(RULE, "fixture", "line-skips"));
    let cases = ["default", "step", "line"].map(|label| {
        (
            label,
            tracked.path().to_path_buf(),
            rules_path(RULE, "configs", &format!("line-skips-{label}.yml")),
        )
    });
    let report = assert_saved_fixture_report(
        "github_actions_download_integrity_line_skips",
        &cases,
        &[],
        &[],
    );
    assert_eq!(report["default"], report["step"]);
    let workflow = ".github/workflows/downloads.yml";
    let quoted = ".github/workflows/quoted.yml";
    assert_finding_locations(
        &report["default"],
        &[(workflow, 28), (workflow, 35), (quoted, 10)],
    );
    assert_finding_locations(
        &report["line"],
        &[
            (".github/actions/download/action.yml", 10),
            (workflow, 9),
            (workflow, 13),
            (workflow, 17),
            (workflow, 22),
            (workflow, 25),
            (workflow, 28),
            (workflow, 33),
            (workflow, 35),
            (".github/workflows/flow.yml", 1),
            (".github/workflows/flow.yml", 1),
            (".github/workflows/folded.yml", 8),
            (".github/workflows/folded.yml", 8),
            (quoted, 10),
            ("scripts/download.sh", 4),
            ("scripts/download.sh", 6),
        ],
    );
}

fn assert_finding_locations(report: &serde_json::Value, expected: &[(&str, u64)]) {
    assert_eq!(report["exitCode"], 1, "{report}");
    let mut actual = report["report"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            assert_eq!(finding["rule"], RULE);
            (
                finding["file"].as_str().unwrap(),
                finding["line"].as_u64().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    actual.sort_unstable();
    assert_eq!(actual, expected);
}
