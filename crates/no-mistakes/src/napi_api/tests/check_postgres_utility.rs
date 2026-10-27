use super::*;

#[test]
fn native_postgres_utility_compatibility_matches_analyze_project() {
    let root = PathBuf::from(fixture("rules", "postgres-require-named-constraints"))
        .join("supported-forms");
    let config = root.join(".no-mistakes.yml");
    let options = json!({"root": root, "config": config});
    let check: serde_json::Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
    )
    .unwrap();
    let project: serde_json::Value = serde_json::from_str(
        &crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(
                json!({
                    "root": root, "config": config, "reports": [{"type": "check"}]
                })
                .to_string(),
            ),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(project["reports"][0]["result"], check);
    assert_eq!(check["rules"], json!([]));
    let advisories = check["advisories"].as_array().unwrap();
    assert_eq!(advisories.len(), 2, "{check}");
    for prefix in ["CREATE EXTENSION", "CREATE USER"] {
        assert!(
            advisories
                .iter()
                .any(|finding| finding["target"].as_str().unwrap().starts_with(prefix)),
            "{check}"
        );
    }
}
