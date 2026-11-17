use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_issue_3426_transaction_lifecycle_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-test-state-mutation",
        "issue-3426",
        "scalar-nullish-transaction-lifecycle",
    );
    let reports = assert_saved_fixture_report(
        "postgres_issue_3426_transaction_lifecycle",
        &[(
            "transaction_lifecycle",
            root.clone(),
            root.join("config.yml"),
        )],
        &[],
        &["--include-sql-verdicts"],
    );
    let report = &reports["transaction_lifecycle"]["report"];
    assert!(
        report["advisories"].as_array().unwrap().is_empty(),
        "{report}"
    );
    let verdicts = report["sqlVerdicts"].as_array().unwrap();
    assert_eq!(verdicts.len(), 5, "{report}");
    let make = verdicts
        .iter()
        .find(|row| row["file"] == "test-helpers/make.mts")
        .unwrap();
    assert_eq!(make["status"], "proven-clear");
    for verdict in verdicts {
        assert_eq!(verdict["evidence"], "exact", "{verdict}");
        assert!(verdict.get("residualReasons").is_none(), "{verdict}");
        let mutation = matches!(
            verdict["file"].as_str(),
            Some("test-helpers/control.mts" | "test-helpers/mutation-tail.mts")
        );
        assert_eq!(
            verdict["status"],
            if mutation {
                "violation"
            } else {
                "proven-clear"
            },
            "{verdict}"
        );
    }
}
