use super::check_json_case::case;
use super::check_json_snapshot::assert_saved_fixture_report;
use super::saved_fixture::materialize_path;
use std::process::Command;

const FIXTURE: &str = "test-cases/rules/shell-compatibility/fixture";

#[test]
fn shell_compatibility_json_matches_snapshot() {
    let (_, source, _) = case("both", FIXTURE, "check.yml");
    let fixture = materialize_path(&source);
    let root = fixture.path();
    let arrays =
        std::fs::read(root.join("legacy/arrays.sh")).expect("read materialized shell script");
    assert!(arrays.contains(&0x0c), "fixture must contain a form feed");
    let cases = [
        ("both", "check.yml"),
        ("custom", "custom.yml"),
        ("no_bash_globs", "no-bash-globs.yml"),
    ]
    .map(|(label, config)| (label, root.to_path_buf(), root.join(config)));
    let report = assert_saved_fixture_report(
        "shell_compatibility",
        &cases,
        &[("PATH", "")],
        &["--timeout", "15"],
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

#[test]
fn shell_compatibility_reports_disappeared_selected_inputs_and_keeps_checking() {
    let (_, root, config) = case("disappeared", FIXTURE, "check.yml");
    let files = [
        "legacy/disappeared.sh",
        "zsh/disappeared.zsh",
        "zsh/disappeared.txt",
        "zsh/startup",
    ]
    .map(|file| root.join(file));
    let findings =
        no_mistakes::codebase::rules::run_filesystem_rules_with_files(&root, Some(&config), &files)
            .expect("check shell compatibility against the supplied file inventory");
    assert_eq!(
        findings
            .iter()
            .map(|finding| (
                finding.rule.as_str(),
                finding.file.as_str(),
                finding.line,
                finding.target.as_deref(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                "bash-empty-array-nounset",
                "legacy/disappeared.sh",
                1,
                Some("unanalyzable"),
            ),
            (
                "zsh-readonly-parameter",
                "zsh/disappeared.zsh",
                1,
                Some("unanalyzable"),
            ),
            ("zsh-readonly-parameter", "zsh/startup", 1, Some("status")),
        ],
    );
}
