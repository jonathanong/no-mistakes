use super::check_json_case::case;
use super::check_json_snapshot::assert_saved_fixture_report;

#[test]
fn shell_compatibility_json_matches_snapshot() {
    let root = "test-cases/rules/shell-compatibility/fixture";
    let report = assert_saved_fixture_report(
        "shell_compatibility",
        &[
            case("both", root, "check.yml"),
            case("custom", root, "custom.yml"),
            case("no_bash_globs", root, "no-bash-globs.yml"),
        ],
        &[("PATH", "")],
        &[],
    );
    assert_eq!(report["both"]["exitCode"], 1);
    assert_eq!(report["custom"]["exitCode"], 1);
    assert_eq!(report["no_bash_globs"]["exitCode"], 0);
}
