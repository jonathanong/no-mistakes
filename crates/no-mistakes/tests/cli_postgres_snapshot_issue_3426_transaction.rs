use super::check_json_snapshot::{assert_saved_fixture_with_env_and_args, rules_path};

#[test]
fn postgres_issue_3426_transaction_lifecycle_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-test-state-mutation",
        "issue-3426",
        "scalar-nullish-transaction-lifecycle",
    );
    assert_saved_fixture_with_env_and_args(
        "postgres_issue_3426_transaction_lifecycle",
        &[("transaction_lifecycle", root.clone(), root.join("config.yml"))],
        &[],
        &["--include-sql-verdicts"],
    );
}
