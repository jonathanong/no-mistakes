use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn shellcheck_config_rejects_invalid_options_before_selection() {
    let root = rules_path("shellcheck-runner", "fixture", "invalid-options");
    let invalid = [
        "severity-blank",
        "severity-unknown",
        "shebang-dir-absolute",
        "shebang-dir-parent",
        "shell-file-absolute",
        "shell-file-directory",
        "shell-file-missing",
        "shell-file-parent",
        "shellcheck-key",
        "skills-lockfile-absolute",
        "skills-lockfile-parent",
    ];
    let cases: Vec<_> = invalid
        .iter()
        .copied()
        .chain(["valid"])
        .map(|name| (name, root.clone(), root.join(format!("{name}.yml"))))
        .collect();
    let output = assert_saved_fixture_report("shellcheck_runner_invalid_options", &cases, &[], &[]);
    for name in invalid {
        assert_eq!(output[name]["exitCode"], 2, "{name}");
        assert!(output[name]["report"].is_null(), "{name}");
    }
    assert_eq!(output["valid"]["exitCode"], 0);
    assert_eq!(output["valid"]["report"]["rules"], serde_json::json!([]));
}
