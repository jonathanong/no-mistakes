use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_lock_written_rows_plain_helper_location_check_json_snapshot() {
    let report = assert_saved_fixture_report(
        "postgres_lock_written_rows_plain_helper_location",
        &[(
            "plain_helper",
            rules_path(
                "postgres-lock-written-rows-only",
                "fixture",
                "plain-helper-location",
            ),
            rules_path(
                "postgres-lock-written-rows-only",
                "configs",
                "plain-helper-location.yml",
            ),
        )],
        &[],
        &[],
    );
    let findings = report["plain_helper"]["report"]["rules"]
        .as_array()
        .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(report["plain_helper"]["report"]["advisories"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(findings[0]["rule"], "postgres-lock-written-rows-only");
    assert_eq!(findings[0]["target"], "constant-advisory-lock");
    assert_eq!(findings[0]["file"], "src/query.ts");
    assert_eq!(findings[0]["line"], 10);
    assert_eq!(
        findings[0]["sourceLine"],
        "await client.query(`SELECT pg_advisory_lock(hashtextextended('k',0))`)"
    );
    assert!(findings[0].get("provenance").is_none());
    assert!(!findings[0].to_string().contains("lockDdl"));
}
