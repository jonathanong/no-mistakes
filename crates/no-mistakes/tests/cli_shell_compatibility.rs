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
fn shell_compatibility_bash_length_guards_match_snapshot() {
    let cases = [
        case(
            "repro",
            "test-cases/rules/shell-compatibility/repro",
            "check.yml",
        ),
        case(
            "guards_and_neighbors",
            "test-cases/rules/shell-compatibility/length-guards",
            "check.yml",
        ),
    ];
    let report = assert_saved_fixture_report("bash_array_length_guards", &cases, &[], &[]);
    assert_eq!(report["repro"]["exitCode"], 0);
    assert_eq!(report["guards_and_neighbors"]["exitCode"], 1);
}

#[test]
fn shell_compatibility_subshell_guards_match_snapshot() {
    let cases = [case(
        "subshell_guards",
        "test-cases/rules/shell-compatibility/subshell-guards",
        "check.yml",
    )];
    let report = assert_saved_fixture_report("bash_subshell_guards", &cases, &[], &[]);
    assert_eq!(report["subshell_guards"]["exitCode"], 1);
}

#[test]
fn shell_compatibility_regression_cases_match_snapshot() {
    let cases = [
        case(
            "regression_cases",
            "test-cases/rules/shell-compatibility/regression-cases",
            "check.yml",
        ),
        case(
            "redirected_function_calls",
            "test-cases/rules/shell-compatibility/redirected-function-calls",
            "check.yml",
        ),
        case(
            "function_scope_arrays",
            "test-cases/rules/shell-compatibility/function-scope-arrays",
            "check.yml",
        ),
        case(
            "function_scope_nounset",
            "test-cases/rules/shell-compatibility/function-scope-nounset",
            "check.yml",
        ),
        case(
            "unused_disable_after_guard",
            "test-cases/rules/shell-compatibility/unused-disable-after-guard",
            "check.yml",
        ),
        case(
            "adversarial_advise",
            "test-cases/rules/shell-compatibility/adversarial-and-unanalyzable",
            "check.yml",
        ),
        case(
            "adversarial_report",
            "test-cases/rules/shell-compatibility/adversarial-and-unanalyzable",
            "check-report.yml",
        ),
        case(
            "correlated_conditions_advise",
            "test-cases/rules/shell-compatibility/correlated-conditions",
            "check.yml",
        ),
        case(
            "correlated_conditions_report",
            "test-cases/rules/shell-compatibility/correlated-conditions",
            "check-report.yml",
        ),
        case(
            "case_scoped_unset_advise",
            "test-cases/rules/shell-compatibility/case-scoped-unset",
            "check.yml",
        ),
        case(
            "case_scoped_unset_report",
            "test-cases/rules/shell-compatibility/case-scoped-unset",
            "check-report.yml",
        ),
    ];
    let report = assert_saved_fixture_report("bash_regression_cases", &cases, &[], &[]);
    assert_eq!(report["regression_cases"]["exitCode"], 1);
    assert_eq!(report["redirected_function_calls"]["exitCode"], 1);
    assert_eq!(report["function_scope_arrays"]["exitCode"], 1);
    assert_eq!(report["function_scope_nounset"]["exitCode"], 1);
    assert_eq!(report["unused_disable_after_guard"]["exitCode"], 1);
    assert_eq!(report["adversarial_advise"]["exitCode"], 1);
    assert_eq!(report["adversarial_report"]["exitCode"], 1);
    assert_eq!(report["correlated_conditions_advise"]["exitCode"], 1);
    assert_eq!(report["correlated_conditions_report"]["exitCode"], 1);
    assert_eq!(report["case_scoped_unset_advise"]["exitCode"], 1);
    assert_eq!(report["case_scoped_unset_report"]["exitCode"], 1);
}

#[test]
fn shell_compatibility_loop_budget_matches_snapshot() {
    let fixture = "test-cases/rules/shell-compatibility/loop-budget";
    let cases = [
        case("advise", fixture, "check.yml"),
        case("report", fixture, "check-report.yml"),
        case(
            "flat",
            "test-cases/rules/shell-compatibility/flat-loop-budget",
            "check.yml",
        ),
    ];
    let report = assert_saved_fixture_report("bash_loop_budget", &cases, &[], &[]);
    assert_eq!(report["advise"]["exitCode"], 1);
    assert_eq!(report["flat"]["exitCode"], 1);
    assert_eq!(report["report"]["exitCode"], 1);
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

#[test]
fn shell_compatibility_custom_message_preserves_unanalyzable_text() {
    let (_, root, config) = case(
        "custom_message",
        "test-cases/rules/shell-compatibility/correlated-conditions",
        "check-custom-message.yml",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["--lock-timeout", "0", "check", "--format", "json", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .output()
        .expect("run custom message shell compatibility check");
    assert!(!output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json output");
    let rules = report["rules"].as_array().expect("rules array");
    assert!(!rules.is_empty());
    assert!(rules
        .iter()
        .all(|item| item["message"] == "Custom empty array message"));
    let advisories = report["advisories"].as_array().expect("advisories array");
    assert!(!advisories.is_empty());
    assert!(advisories.iter().all(|item| {
        item["message"]
            .as_str()
            .is_some_and(|msg| msg.starts_with("Cannot prove bash array safety:"))
    }));
}
