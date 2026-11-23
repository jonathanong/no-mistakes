use super::check_json_case::case;
use super::check_json_snapshot::assert_saved_fixture_report;
use std::process::Command;

const FIXTURE: &str = "test-cases/rules/shell-compatibility/fixture";

#[test]
fn shell_compatibility_json_matches_snapshot() {
    let report = assert_saved_fixture_report(
        "shell_compatibility",
        &[
            case("both", FIXTURE, "check.yml"),
            case("custom", FIXTURE, "custom.yml"),
            case("no_bash_globs", FIXTURE, "no-bash-globs.yml"),
        ],
        &[("PATH", "")],
        &[],
    );
    assert_eq!(report["both"]["exitCode"], 1);
    assert_eq!(report["custom"]["exitCode"], 1);
    assert_eq!(report["no_bash_globs"]["exitCode"], 0);
}

#[test]
fn shell_compatibility_rejects_invalid_file_globs() {
    let (_, root, config) = case("invalid_glob", FIXTURE, "invalid-glob.yml");
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["--lock-timeout", "0", "check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .output()
        .expect("run shell compatibility check");
    assert!(!output.status.success());
    let diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        diagnostic.contains("bash-empty-array-nounset options.files"),
        "{diagnostic}"
    );
}
