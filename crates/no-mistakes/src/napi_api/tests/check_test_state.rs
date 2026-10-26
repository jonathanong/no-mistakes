use super::*;

#[test]
fn check_json_exposes_test_sql_mutations_and_shared_suppressions() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-test-state-mutation/fixture");
    let output = check_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "config": root.join(".no-mistakes.yml"),
            "includeSuppressed": true,
            "includeSqlVerdicts": true,
        })
        .to_string(),
    ))
    .unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    let findings = report["rules"].as_array().unwrap();
    for target in [
        "ALTER TABLE",
        "CREATE VIEW",
        "pg_sleep",
        "session_replication_role",
    ] {
        assert!(
            findings.iter().any(
                |finding| finding["rule"] == "postgres-no-test-state-mutation"
                    && finding["target"] == target
            ),
            "{report}"
        );
    }
    assert!(findings
        .iter()
        .all(|finding| !finding["file"].as_str().unwrap().contains("suppressed")));
    let suppressed = report["suppressed"].as_array().unwrap();
    assert_eq!(suppressed.len(), 3, "{report}");
    for kind in ["file", "line", "nextLine"] {
        assert!(
            suppressed
                .iter()
                .any(|finding| finding["directive"]["kind"] == kind),
            "{report}"
        );
    }
}
