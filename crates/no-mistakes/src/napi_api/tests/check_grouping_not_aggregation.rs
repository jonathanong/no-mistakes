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

#[test]
fn grouping_sql_policy_aliases_match_check_and_analyze_project() {
    let root =
        PathBuf::from(fixture("rules", "postgres-grouping-not-aggregation")).join("unanalyzable");
    for (policy, count) in [
        ("default", 2),
        ("rule-fail", 2),
        ("shared-report", 2),
        ("rule-report", 2),
        ("rule-sql-report", 2),
        ("shared-ignore-rule-sql-report", 2),
        ("shared-sql-report", 2),
        ("project-fail", 2),
        ("rule-ignore-over-fail", 1),
    ] {
        let options = json!({"root": root, "config": root.join(format!("{policy}.yml"))});
        let check: serde_json::Value = serde_json::from_str(
            &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
        )
        .unwrap();
        let mut project_options = options;
        project_options["reports"] = json!([{"type": "check"}]);
        let project: serde_json::Value = serde_json::from_str(
            &crate::napi_api::analyze_project::analyze_project_json_impl(
                crate::napi_api::options::test_json_arg(project_options.to_string()),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(project["reports"][0]["result"], check, "{policy}");
        assert_eq!(check["rules"], json!([]), "{policy}: {check}");
        assert_eq!(check["warnings"], json!([]), "{policy}: {check}");
        let advisories = check["advisories"].as_array().unwrap();
        assert_eq!(advisories.len(), count, "{policy}: {check}");
        assert!(
            advisories.iter().any(|finding| {
                finding["target"] == "distinct-over-unique-key" && finding["line"] == 4
            }),
            "{policy}: {check}"
        );
        assert_eq!(
            advisories
                .iter()
                .filter(|finding| finding["line"] == 5)
                .count(),
            count - 1,
            "{policy}: {check}"
        );
    }
}
