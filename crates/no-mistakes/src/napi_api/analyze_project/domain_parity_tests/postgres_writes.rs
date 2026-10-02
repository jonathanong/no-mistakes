#[test]
fn prepared_postgres_generated_write_check_matches_standalone() {
    let root = repo_fixture(&["test-cases", "rules", "postgres-no-generated-column-writes", "unit-fixture", "do-write-lines"]);
    let standalone = parse_json(crate::napi_api::check_json_impl(
        crate::napi_api::options::test_json_arg(json!({ "root": root, "config": ".no-mistakes.yml" }))
    ).unwrap());
    let aggregate = parse_json(analyze_project_json_impl(
        crate::napi_api::options::test_json_arg(json!({ "root": root, "config": ".no-mistakes.yml", "reports": [{ "type": "check" }] }))
    ).unwrap());
    assert_eq!(standalone["rules"].as_array().unwrap().iter().map(|finding| finding["line"].as_u64().unwrap()).collect::<Vec<_>>(), [7, 8]);
    assert_eq!(aggregate["reports"][0]["result"], standalone);
}
