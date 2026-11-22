use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};
use super::cli_postgres_snapshot_lock_written_rows::snapshot;

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

#[test]
fn postgres_lock_written_rows_ddl_check_json_snapshot() {
    snapshot(
        "postgres_lock_written_rows_ddl",
        &[
            ("related", "ddl-related", "checks-fail.yml"),
            ("guard", "ddl-guard", "checks-fail.yml"),
            ("catalog", "ddl-catalog", "ddl-catalog.yml"),
            ("negative", "ddl-negative", "checks-fail.yml"),
            ("uncertain", "ddl-uncertain", "checks-fail.yml"),
            ("dynamic_identifiers", "ddl-dynamic", "ddl-dynamic.yml"),
        ],
    );
}

#[test]
fn postgres_lock_written_rows_trigger_writes_check_json_snapshot() {
    snapshot(
        "postgres_lock_written_rows_trigger_writes",
        &[
            ("trigger_effects", "trigger-writes", "trigger-writes.yml"),
            ("without_catalog", "trigger-writes", "checks-fail.yml"),
            (
                "excluded_catalog",
                "trigger-writes",
                "trigger-catalog-excluded.yml",
            ),
            (
                "missing_catalog",
                "trigger-writes",
                "trigger-catalog-missing.yml",
            ),
            ("depth_bound", "trigger-depth", "trigger-writes.yml"),
            ("upsert", "trigger-upsert", "trigger-writes.yml"),
            ("composite", "trigger-composite", "trigger-writes.yml"),
            (
                "dynamic_setting",
                "trigger-dynamic-setting",
                "trigger-writes.yml",
            ),
            (
                "replication_role",
                "trigger-replication-role",
                "trigger-writes.yml",
            ),
        ],
    );
}

#[test]
fn postgres_lock_written_rows_parameter_transactions_check_json_snapshot() {
    let report = snapshot(
        "postgres_lock_written_rows_parameter_transactions",
        &[
            (
                "disabled",
                "parameter-transactions",
                "parameter-transactions-disabled.yml",
            ),
            (
                "enabled",
                "parameter-transactions",
                "parameter-transactions.yml",
            ),
            (
                "no_catalog",
                "parameter-transactions",
                "parameter-transactions-no-catalog.yml",
            ),
        ],
    );
    let advisories = report["enabled"]["report"]["advisories"]
        .as_array()
        .unwrap();
    let has_advisory = |line: u64| advisories.iter().any(|advisory| advisory["line"] == line);
    for line in [5, 12, 97, 102, 130, 170, 180, 185] {
        assert!(!has_advisory(line), "expected a proof for line {line}");
    }
    for line in [
        17, 21, 107, 112, 117, 123, 136, 143, 149, 155, 160, 165, 175, 191, 198,
    ] {
        assert!(has_advisory(line), "expected an abstention for line {line}");
    }
    let advisory_lines: Vec<_> = report["no_catalog"]["report"]["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|advisory| advisory["line"].as_u64().unwrap())
        .collect();
    for line in [5, 175, 191, 198] {
        assert!(
            advisory_lines.contains(&line),
            "expected an abstention for line {line}"
        );
    }
    for line in [130, 180, 185, 203, 208, 213] {
        assert!(
            !advisory_lines.contains(&line),
            "expected a proof for line {line}"
        );
    }
}
