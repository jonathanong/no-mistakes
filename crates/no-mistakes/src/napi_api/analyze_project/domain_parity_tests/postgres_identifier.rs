#[test]
fn prepared_migration_identifier_check_matches_standalone() {
    let root = repo_fixture(&["test-cases", "rules", "postgres-identifier-length", "fixture", "fail"]);
    let standalone = parse_json(crate::napi_api::check_json_impl(
        crate::napi_api::options::test_json_arg(json!({ "root": root, "config": ".no-mistakes.yml" }))
    ).unwrap());
    let aggregate = parse_json(analyze_project_json_impl(
        crate::napi_api::options::test_json_arg(json!({
            "root": root,
            "config": ".no-mistakes.yml",
            "reports": [{ "type": "check" }]
        }))
    ).unwrap());
    assert!(standalone["rules"].as_array().unwrap().iter().any(|finding|
        finding["rule"] == "postgres-identifier-length" && finding["target"].as_str().is_some_and(|name| name.starts_with("added_inline_constraint"))
    ));
    assert_eq!(aggregate["reports"][0]["result"], standalone);
}

