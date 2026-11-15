use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_closed_identifier_correlation_check_json_snapshot() {
    let root = rules_path(
        "postgres-bounded-statements",
        "fixture",
        "closed-identifier-correlation",
    );
    let cases = [("correlation", root.clone(), root.join(".no-mistakes.yml"))];
    let report =
        assert_saved_fixture_report("postgres_closed_identifier_correlation", &cases, &[], &[]);
    let rows = report["correlation"]["report"]["rules"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["target"], "table:posts, table:users");
    assert!(
        rows[0]["sourceLine"]
            .as_str()
            .is_some_and(|line| line.contains("column(other)")),
        "only the uncorrelated call may report: {rows:?}"
    );
}
