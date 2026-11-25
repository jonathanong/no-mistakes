#[test]
fn prepared_explain_write_rollback_check_matches_standalone() {
    let root = repo_fixture(&[
        "test-cases",
        "rules",
        "postgres-explain-write-rollback",
        "fixture",
        "example",
    ]);
    let standalone = parse_json(
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(json!({
            "root": root, "config": ".no-mistakes.yml"
        })))
        .unwrap(),
    );
    let aggregate = parse_json(
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(json!({
            "root": root, "config": ".no-mistakes.yml", "reports": [{ "type": "check" }]
        })))
        .unwrap(),
    );
    let findings = standalone["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["rule"], "postgres-explain-write-rollback");
    assert_eq!(findings[0]["file"], "benchmarks/update.sql");
    assert_eq!(findings[0]["line"], 1);
    assert_eq!(findings[0]["target"], "UPDATE");
    assert_eq!(aggregate["reports"][0]["result"], standalone);
}
