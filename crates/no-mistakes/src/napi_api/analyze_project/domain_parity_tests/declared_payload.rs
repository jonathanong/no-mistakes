#[test]
fn declared_payload_checks_match_standalone_async_and_prepared_project_reports() {
    for (name, count) in [("pass", 0), ("fail", 1), ("suppression", 1)] {
        let source = repo_fixture(&["fixtures", "rules", "declared-payload-compatibility", name]);
        let fixture = crate::test_support::materialize_saved_fixture(&source);
        let root = fixture.path().canonicalize().unwrap();
        let standalone = parse_json(crate::napi_api::check_json_impl(
            crate::napi_api::options::test_json_arg(json!({"root": root})),
        ).unwrap());
        let aggregate = parse_json(analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(json!({
                "root": root, "reports": [{"type": "check"}]
            })),
        ).unwrap());
        assert_eq!(aggregate["reports"][0]["result"], standalone);
        assert_eq!(standalone["rules"].as_array().unwrap().len(), count, "{standalone}");
    }
}
