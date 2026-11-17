use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_lock_written_rows_catalog_reconciliation_check_json_snapshot() {
    let rule = "postgres-lock-written-rows-only";
    let root = rules_path(rule, "fixture", "mixed-ddl-trigger");
    let cases = [
        ("complete", "mixed-ddl-trigger-complete.yml"),
        ("ordering", "mixed-ddl-trigger-ordering.yml"),
        ("ignored_check", "mixed-ddl-trigger-ignored-check.yml"),
    ]
    .map(|(name, config)| (name, root.clone(), rules_path(rule, "configs", config)));
    let report = assert_saved_fixture_report(
        "postgres_lock_written_rows_catalog_reconciliation",
        &cases,
        &[],
        &[],
    );
    assert_eq!(report["complete"]["exitCode"], 0);
    assert_eq!(
        report["ordering"]["report"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(report["ordering"]["report"]["rules"][0]["line"], 11);
    assert_eq!(report["ignored_check"]["exitCode"], 0);
}
