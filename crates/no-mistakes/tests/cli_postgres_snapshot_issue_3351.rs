use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn forwarded_executor_wrappers_check_json_snapshot() {
    let cases: Vec<_> = [
        "forwarding",
        "non-forwarding",
        "duplicate",
        "destructured",
        "early-call",
        "early-alias",
        "early-member",
        "later-escapes",
    ]
    .into_iter()
    .map(|label| {
        (
            label,
            rules_path(
                "postgres-no-offset",
                "fixture",
                &format!("issue-3351/{label}"),
            ),
            rules_path("postgres-no-offset", "configs", "issue-3351.yml"),
        )
    })
    .collect();
    let mut cases = cases;
    cases.push((
        "transaction",
        rules_path("postgres-no-offset", "fixture", "issue-3351/transaction"),
        rules_path(
            "postgres-no-offset",
            "configs",
            "issue-3351-transaction.yml",
        ),
    ));
    assert_saved_fixture("forwarded_executor_wrappers", &cases);
}
