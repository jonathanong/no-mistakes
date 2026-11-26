use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-no-required-pr-path-filters";

#[test]
fn github_actions_required_pr_path_filters_json_matches_snapshot() {
    let root = rules_path(RULE, "fixture", "");
    let cases = [
        "default",
        "disabled",
        "missing",
        "excluded",
        "advisory",
        "aliased",
        "unrecognised",
        "unrecognised-advisory",
    ]
    .map(|case| {
        (
            case,
            root.clone(),
            rules_path(RULE, "configs", &format!("{case}.yml")),
        )
    });
    assert_saved_fixture("github_actions_required_pr_path_filters", &cases);
}

#[test]
fn required_pr_path_filters_rejects_missing_empty_and_invalid_workflows() {
    let root = rules_path(RULE, "fixture", "");
    for (case, expected_snippet) in [
        ("missing-workflows", "[]"),
        ("empty-workflows", "[]"),
        ("invalid", ".github/workflows/*.yml"),
    ] {
        let config = rules_path(RULE, "configs", &format!("{case}.yml"));
        let run = |jobs: &str, json: bool| {
            let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
            cmd.args(["--lock-timeout", "0", "--jobs", jobs, "check", "--root"])
                .arg(&root)
                .arg("--config")
                .arg(&config);
            if json {
                cmd.args(["--format", "json"]);
            }
            cmd.output().unwrap()
        };
        let one = run("1", true);
        let two = run("2", true);
        let human = run("1", false);
        assert_eq!(one.status.code(), Some(2), "{case}");
        assert_eq!(two.status.code(), Some(2), "{case}");
        assert_eq!(human.status.code(), Some(2), "{case}");
        assert!(one.stdout.is_empty(), "{case} stdout must be empty");
        assert!(two.stdout.is_empty(), "{case} stdout must be empty");
        assert!(human.stdout.is_empty(), "{case} human stdout must be empty");
        let stderr = String::from_utf8(one.stderr).unwrap();
        assert_eq!(stderr, String::from_utf8(two.stderr).unwrap(), "{case}");
        assert_eq!(stderr, String::from_utf8(human.stderr).unwrap(), "{case}");
        assert!(stderr.contains(RULE), "{case}: {stderr}");
        assert!(stderr.contains("options.workflows"), "{case}: {stderr}");
        assert!(stderr.contains(expected_snippet), "{case}: {stderr}");
    }
}

#[test]
fn standalone_required_pr_check_preserves_workflow_targets() {
    use no_mistakes::codebase::rules::github_actions_no_required_pr_path_filters::check;
    use no_mistakes::config::v2::discover::load_v2_config;

    let root = rules_path(RULE, "fixture", "");
    let config = rules_path(RULE, "configs", "default.yml");
    let config = load_v2_config(&root, Some(&config)).expect("load saved rule configuration");
    let findings = check(&root, &config).expect("check saved workflows");
    let mut targets: Vec<_> = findings
        .iter()
        .filter_map(|finding| finding.target.as_deref())
        .collect();
    targets.sort_unstable();
    assert_eq!(
        targets,
        [
            "on.pull_request.paths",
            "on.pull_request_target.paths-ignore",
            "unanalyzable",
        ]
    );
}
