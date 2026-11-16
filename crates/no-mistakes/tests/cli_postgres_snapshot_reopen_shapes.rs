use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};
use std::path::PathBuf;

#[path = "cli_postgres_snapshot_reopen_shapes/bind_sequence.rs"]
mod bind_sequence;

fn case_root(rule: &str, name: &str) -> PathBuf {
    let mut path = rules_path(rule, name, "src");
    path.pop();
    path
}

fn scalar_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/parameterized-imported-values")
}

#[test]
fn postgres_scalar_value_contexts_check_json_snapshot() {
    let root = scalar_root();
    let reports = assert_saved_fixture_report(
        "postgres_scalar_value_contexts",
        &[("values", root.clone(), root.join("config.yml"))],
        &[],
        &["--include-sql-verdicts"],
    );
    let report = &reports["values"]["report"];
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 5, "{report}");
    assert!(findings.iter().all(|row| {
        row["rule"] == "postgres-no-offset"
            && row["target"].as_str().unwrap().split('#').next() == Some("offset")
    }));
    let verdicts = report["sqlVerdicts"].as_array().unwrap();
    assert_eq!(verdicts.len(), 5, "{report}");
    assert!(verdicts.iter().all(|row| {
        row["status"] == "violation"
            && row["evidence"] == "exact"
            && row.get("residualReasons").is_none()
    }), "{report}");
}

#[test]
fn postgres_reopen_shapes_check_json_snapshot() {
    let reports = assert_saved_fixture_report(
        "postgres_reopen_shapes",
        &[
            (
                "cross_module_claim",
                case_root("postgres-lock-ordering", "cross-module-claim"),
                rules_path("postgres-lock-ordering", "cross-module-claim", "config.yml"),
            ),
            (
                "statement_append",
                case_root("postgres-lock-ordering", "statement-append"),
                rules_path("postgres-lock-ordering", "statement-append", "config.yml"),
            ),
            (
                "raw_column_append",
                case_root("postgres-required-predicates", "raw-column-append"),
                rules_path(
                    "postgres-required-predicates",
                    "raw-column-append",
                    "config.yml",
                ),
            ),
            (
                "imported_limit",
                case_root("postgres-conflict-ordering", "imported-limit"),
                rules_path("postgres-conflict-ordering", "imported-limit", "config.yml"),
            ),
        ],
        &[],
        &["--include-sql-verdicts"],
    );
    for (case, rule, expected) in [
        ("cross_module_claim", "postgres-lock-ordering", 1),
        ("statement_append", "postgres-lock-ordering", 1),
        ("raw_column_append", "postgres-required-predicates", 1),
        ("imported_limit", "postgres-conflict-ordering", 2),
    ] {
        let report = &reports[case]["report"];
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), expected, "{case}: {report}");
        assert!(findings.iter().all(|finding| finding["rule"] == rule));
        assert!(
            report["warnings"].as_array().unwrap().is_empty(),
            "{report}"
        );
        assert!(!report.to_string().contains("unused-disable"), "{report}");
        let verdicts = report["sqlVerdicts"].as_array().unwrap();
        assert!(!verdicts.is_empty(), "{case}: {report}");
        assert!(verdicts.iter().any(|row| row["status"] == "violation"));
        assert!(
            verdicts.iter().all(|row| {
                row["status"] != "abstain"
                    && (row["evidence"] == "exact" || row["evidence"] == "exhaustive-union")
                    && row.get("residualReasons").is_none()
            }),
            "{case}: {report}"
        );
        if case == "imported_limit" {
            assert_eq!(verdicts.len(), 4, "{report}");
            for file in ["src/insert.ts", "src/root-cause.ts"] {
                assert_eq!(
                    findings.iter().filter(|row| row["file"] == file).count(),
                    1,
                    "{report}"
                );
            }
            assert!(verdicts
                .iter()
                .all(|row| { row["evidence"] == "exact" && row["status"] == "violation" }));
        } else if case == "raw_column_append" {
            assert_eq!(verdicts.len(), 2, "{report}");
            assert!(verdicts.iter().all(|row| row["status"] == "violation"));
        } else {
            assert!(verdicts.iter().any(|row| row["status"] == "proven-clear"));
        }
        for finding in findings {
            let expected_target = match case {
                "raw_column_append" => "candidates",
                "imported_limit" if finding["file"] == "src/root-cause.ts" => "targetless-arbiter",
                "imported_limit" => "missing-canonical-order",
                _ => "lock-ordering",
            };
            assert_eq!(finding["target"], expected_target, "{case}: {finding}");
        }
    }
}
