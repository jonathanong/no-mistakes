use super::*;

#[test]
fn grouping_advice_is_non_failing_and_matches_analyze_project() {
    let root = PathBuf::from(fixture("rules", "postgres-grouping-not-aggregation"));
    let options = json!({"root": root});
    let check: serde_json::Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
    )
    .unwrap();
    let project: serde_json::Value = serde_json::from_str(
        &crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(
                json!({"root": root, "reports": [{"type": "check"}]}).to_string(),
            ),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(project["reports"][0]["result"], check);
    assert_eq!(check["rules"], json!([]), "{check}");
    assert_eq!(check["warnings"], json!([]), "{check}");
    assert!(
        check["advisories"].as_array().is_some_and(|rows| {
            rows.iter().any(|row| {
                row["rule"] == "postgres-grouping-not-aggregation"
                    && row["file"] == "raw/shape-controls.sql"
                    && row["target"] == "all-of-candidate-pool"
            })
        }),
        "{check}"
    );
}
