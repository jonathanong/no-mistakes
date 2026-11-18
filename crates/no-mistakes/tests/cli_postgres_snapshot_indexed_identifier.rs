use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_indexed_identifier_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3598",
    );
    let report = assert_saved_fixture_report(
        "postgres_indexed_identifier",
        &[("max", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let findings = report["max"]["report"]["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 26);
    assert!(findings
        .iter()
        .all(|finding| finding["file"] == "negative.ts"
            || finding["file"] == "mutations.ts"
            || finding["file"] == "class-shadow.ts"));
    assert_eq!(report["max"]["report"]["warnings"], serde_json::json!([]));
    assert_eq!(report["max"]["report"]["advisories"], serde_json::json!([]));
}
