use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn transaction_control_statement_policy_check_json_snapshot() {
    let rule = "postgres-sql-statement-policy";
    let root = rules_path(rule, "fixture", "transaction-control");
    let cases = [
        ("group", "txn-control-group.yml"),
        ("begin_commit", "txn-control-begin-commit.yml"),
        ("only_begin", "txn-control-only-begin.yml"),
        (
            "only_start_transaction",
            "txn-control-only-start-transaction.yml",
        ),
        ("only_commit", "txn-control-only-commit.yml"),
        ("only_end", "txn-control-only-end.yml"),
        ("only_rollback", "txn-control-only-rollback.yml"),
        ("only_savepoint", "txn-control-only-savepoint.yml"),
        (
            "only_release_savepoint",
            "txn-control-only-release-savepoint.yml",
        ),
        (
            "only_rollback_to_savepoint",
            "txn-control-only-rollback-to-savepoint.yml",
        ),
        ("embedded", "txn-control-embedded.yml"),
    ]
    .map(|(label, config)| (label, root.clone(), rules_path(rule, "configs", config)));
    let reports = assert_saved_fixture_report(
        "postgres_sql_statement_policy_transaction_control",
        &cases,
        &[],
        &[],
    );

    for (label, _, _) in cases {
        let case = &reports[label];
        assert_eq!(case["exitCode"], 1, "{label}: {case}");
        let findings = case["report"]["rules"].as_array().unwrap();
        assert!(!findings.is_empty(), "{label}: {case}");
        assert!(
            findings.iter().all(|finding| {
                finding["rule"] == rule
                    && finding["file"] != "scripts/manual_backfill.sql"
                    && finding["message"]
                        .as_str()
                        .unwrap()
                        .contains("migration runner already owns the transaction")
            }),
            "{label}: {case}"
        );
    }

    let group_findings = reports["group"]["report"]["rules"].as_array().unwrap();
    assert_eq!(group_findings.len(), 10, "{group_findings:?}");
    assert_eq!(
        reports["group"]["report"]["advisories"],
        serde_json::json!([])
    );

    let begin_commit = reports["begin_commit"]["report"]["rules"]
        .as_array()
        .unwrap();
    let mut begin_commit_sites: Vec<_> = begin_commit
        .iter()
        .map(|f| {
            (
                f["file"].as_str().unwrap(),
                f["line"].as_u64().unwrap(),
                f["target"].as_str().unwrap(),
            )
        })
        .collect();
    begin_commit_sites.sort_unstable();
    assert_eq!(
        begin_commit_sites,
        [
            ("migrations/001_begin_commit.sql", 1, "BEGIN"),
            ("migrations/001_begin_commit.sql", 14, "COMMIT"),
            ("migrations/002_all_kinds.sql", 5, "BEGIN"),
            ("migrations/002_all_kinds.sql", 10, "COMMIT"),
        ]
    );

    for (label, kind, expected_count) in [
        ("only_begin", "BEGIN", 2),
        ("only_start_transaction", "START TRANSACTION", 1),
        ("only_commit", "COMMIT", 2),
        ("only_end", "END", 1),
        ("only_rollback", "ROLLBACK", 1),
        ("only_savepoint", "SAVEPOINT", 1),
        ("only_release_savepoint", "RELEASE SAVEPOINT", 1),
        ("only_rollback_to_savepoint", "ROLLBACK TO SAVEPOINT", 1),
    ] {
        let findings = reports[label]["report"]["rules"].as_array().unwrap();
        assert_eq!(findings.len(), expected_count, "{label}: {findings:?}");
        assert!(
            findings.iter().all(|f| f["target"] == kind),
            "{label}: {findings:?}"
        );
    }

    let embedded_findings = reports["embedded"]["report"]["rules"].as_array().unwrap();
    assert_eq!(
        embedded_findings
            .iter()
            .filter(|f| f["file"] == "src/runner.ts")
            .map(|f| (f["line"].as_u64().unwrap(), f["target"].as_str().unwrap()))
            .collect::<Vec<_>>(),
        [(4, "BEGIN"), (6, "COMMIT")]
    );
}
