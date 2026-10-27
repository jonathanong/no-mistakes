use super::check_json_case::case;
use super::check_json_snapshot::assert_saved_fixture_with_env;

// ShellCheck output depends on the host, so these cases run with an empty PATH:
// the tool is deterministically absent and the rule routes through its policy.
#[test]
fn shellcheck_runner_json_matches_snapshot() {
    let root = "test-cases/rules/shellcheck-runner/fixture/unavailable";
    let cases: Vec<_> = [
        ("report", "report.yml"),
        ("advise", "advise.yml"),
        ("ignore", "ignore.yml"),
        ("no_selected", "no-selected.yml"),
        ("rule_ignore", "rule-ignore.yml"),
        ("scoped", "scoped.yml"),
        ("custom", "custom.yml"),
    ]
    .into_iter()
    .map(|(label, config)| case(label, root, config))
    .collect();
    assert_saved_fixture_with_env("shellcheck_runner", &cases, &[("PATH", "")]);
}
