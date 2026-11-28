#[test]
fn prepared_lock_reason_check_matches_standalone() {
    let root = repo_fixture(&[
        "test-cases",
        "rules",
        "postgres-lock-reason",
        "fixture",
        "advisory",
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
    assert!(standalone["rules"].as_array().unwrap().is_empty());
    let advisories = standalone["advisories"].as_array().unwrap();
    assert!(!advisories.is_empty());
    assert!(advisories
        .iter()
        .all(|item| item["rule"] == "postgres-lock-reason"));
    assert_eq!(aggregate["reports"][0]["result"], standalone);
}
