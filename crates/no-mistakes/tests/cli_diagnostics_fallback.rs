use super::check_json_case::case;
use super::check_json_snapshot::assert_saved_fixture;
use std::process::Command;

#[test]
fn check_json_snapshot_diagnostics_fallback_preserves_plain_report() {
    let saved = case(
        "fallback",
        "test-cases/codebase-analysis/diagnostics-fallback/fixture",
        ".no-mistakes.yml",
    );
    let disabled = case(
        "disabled",
        "test-cases/codebase-analysis/diagnostics-fallback/fixture",
        "disabled.yml",
    );
    assert_saved_fixture("diagnostics_fallback", &[saved.clone(), disabled]);
    let run = |verbose| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
        command
            .args(["check", "--root"])
            .arg(&saved.1)
            .arg("--config")
            .arg(&saved.2)
            .args(["--format", "json"]);
        if verbose {
            command.arg("--verbose-timings");
        }
        command.output().unwrap()
    };
    let plain = run(false);
    let verbose = run(true);
    assert_eq!(plain.status.code(), verbose.status.code());
    assert_eq!(plain.stdout, verbose.stdout);
    let plain_stderr = String::from_utf8(plain.stderr).unwrap();
    assert!(!plain_stderr.lines().any(is_diagnostic));
    let stderr = String::from_utf8(verbose.stderr).unwrap();
    assert_eq!(
        stderr
            .lines()
            .filter(|line| !is_diagnostic(line))
            .collect::<Vec<_>>(),
        plain_stderr.lines().collect::<Vec<_>>()
    );
    for pass in ["value_flow.extract", "value_flow.finalize"] {
        assert!(
            stderr
                .lines()
                .any(|line| line.starts_with(&format!("[hot] {pass}: "))
                    && line.contains("legacy.source")),
            "{stderr}"
        );
    }
}

fn is_diagnostic(line: &str) -> bool {
    ["[hot] ", "[cpu] ", "[memory] ", "[timing] ", "[work] "]
        .iter()
        .any(|prefix| line.starts_with(prefix))
}
