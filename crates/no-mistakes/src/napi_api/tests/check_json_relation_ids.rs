use super::*;

#[test]
fn json_relation_id_findings_are_non_failing_and_match_analyze_project() {
    let root = PathBuf::from(fixture("rules", "postgres-json-relation-ids"));
    let config = root.join("no-mistakes.max.yml");
    let options = json!({"root": root, "config": config});
    let check: serde_json::Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
    )
    .unwrap();
    let project: serde_json::Value = serde_json::from_str(
        &crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(
                json!({
                    "root": root,
                    "config": config,
                    "reports": [{"type": "check"}]
                })
                .to_string(),
            ),
        )
        .unwrap(),
    )
    .unwrap();
    let result = &project["reports"][0]["result"];

    assert_eq!(check["rules"], json!([]), "{check}");
    assert_eq!(check["warnings"], json!([]), "{check}");
    assert_eq!(*result, check);
    assert!(check["advisories"].as_array().is_some_and(|rows| {
        rows.iter().any(|finding| {
            finding
                == &json!({
                    "rule": "postgres-json-relation-ids",
                    "file": "db/fail.sql",
                    "line": 3,
                    "message": "db/fail.sql:3: key \"accountId\" written into orders.details has no same-row foreign key column; write account_id with a foreign key, or allow the column with a reason",
                    "target": "column:orders.details"
                })
        })
    }), "expected the exact SQL advisory DTO: {check}");
}
