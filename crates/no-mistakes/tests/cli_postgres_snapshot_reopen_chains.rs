use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};
use std::path::PathBuf;

fn case_root(rule: &str, name: &str) -> PathBuf {
    let mut path = rules_path(rule, name, "src");
    path.pop();
    path
}

#[test]
fn postgres_reopen_chains_check_json_snapshot() {
    let reports = assert_saved_fixture_report(
        "postgres_reopen_chains",
        &[
            (
                "positional_parameters",
                case_root("postgres-lock-ordering", "positional-parameters"),
                rules_path(
                    "postgres-lock-ordering",
                    "positional-parameters",
                    "config.yml",
                ),
            ),
            (
                "append_chains",
                case_root("postgres-lock-ordering", "append-chains"),
                rules_path("postgres-lock-ordering", "append-chains", "config.yml"),
            ),
            (
                "numeric_const_limit",
                case_root("postgres-conflict-ordering", "numeric-const-limit"),
                rules_path(
                    "postgres-conflict-ordering",
                    "numeric-const-limit",
                    "config.yml",
                ),
            ),
        ],
        &[],
        &[],
    );
    for (case, rule, target) in [
        (
            "positional_parameters",
            "postgres-lock-ordering",
            "lock-ordering",
        ),
        ("append_chains", "postgres-lock-ordering", "lock-ordering"),
        (
            "numeric_const_limit",
            "postgres-conflict-ordering",
            "targetless-arbiter",
        ),
    ] {
        let findings = reports[case]["report"]["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 1, "{case}: {reports}");
        assert_eq!(findings[0]["file"], "src/violation.ts", "{case}");
        assert_eq!(findings[0]["rule"], rule, "{case}");
        assert_eq!(findings[0]["target"], target, "{case}");
    }
}
