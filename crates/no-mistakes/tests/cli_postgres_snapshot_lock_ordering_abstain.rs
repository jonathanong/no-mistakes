use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_lock_ordering_abstain_check_json_snapshot() {
    let rule = "postgres-lock-ordering";
    let reports = assert_saved_fixture_report(
        "postgres_lock_ordering_abstain",
        &[
            (
                "closed_table_identifier",
                rules_path(rule, "fixture", "closed-table-identifier"),
                rules_path(rule, "configs", "catalog.yml"),
            ),
            (
                "appended_scope_fragment",
                rules_path(rule, "fixture", "appended-scope-fragment"),
                rules_path(rule, "configs", "ignore-unanalyzable.yml"),
            ),
        ],
        &[],
        &[],
    );
    let findings = reports["appended_scope_fragment"]["report"]["rules"]
        .as_array()
        .unwrap();
    assert_eq!(findings.len(), 1, "{reports}");
    assert_eq!(findings[0]["file"], "src/violation.ts");
    assert_eq!(findings[0]["target"], "lock-ordering");
}
