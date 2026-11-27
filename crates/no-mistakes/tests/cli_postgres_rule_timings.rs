use super::check_json_snapshot::rules_path;
use std::process::Command;

#[test]
fn postgres_graph_rule_timings_use_rule_ids() {
    let cases = [
        (
            "postgres-no-mutable-sql-registry",
            "fixture",
            "fail-member-write",
        ),
        (
            "postgres-no-unbounded-sql-identifier",
            "fixture",
            "fail-env",
        ),
        (
            "postgres-sql-fragment-escape",
            "fixtures",
            "import-resolution",
        ),
    ];
    for (rule, directory, fixture) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args([
                "--jobs",
                "1",
                "--lock-timeout",
                "0",
                "--verbose-timings",
                "check",
                "--root",
            ])
            .arg(rules_path(rule, directory, fixture))
            .args(["--format", "json"])
            .output()
            .expect("run PostgreSQL graph rule");
        let diagnostics = String::from_utf8(output.stderr).expect("diagnostics");
        assert_eq!(output.status.code(), Some(1), "{rule}: {diagnostics}");
        let expected = format!("[timing] analysis.rules.{rule}: ");
        assert!(
            diagnostics.lines().any(|line| line.starts_with(&expected)),
            "{rule}: expected {expected:?} in {diagnostics}"
        );
    }
}
