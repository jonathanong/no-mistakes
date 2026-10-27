use super::check_json_case::case;
use super::check_json_snapshot::assert_saved_fixture_with_env;

// ShellCheck output depends on the host, so these cases run with an empty PATH:
// the tool is deterministically absent and the rule routes through its policy.
// An empty PATH entry resolves to the current directory on Unix, and Windows
// searches the current directory too, so the fixture directory (the child's cwd)
// must never contain a `shellcheck` binary or the tool would no longer be absent.
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
