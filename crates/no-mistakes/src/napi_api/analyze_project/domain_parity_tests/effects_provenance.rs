
#[test]
fn effects_provenance_is_shared_by_standalone_and_aggregate_apis() {
    let root = repo_fixture(&["test-cases", "rules", "query-reached-per-item", "provenance-fixture"]);
    let standalone = parse_json(crate::napi_api::effects_json_impl(crate::napi_api::options::test_json_arg(json!({
        "root": root, "kind": "valkey", "entry": "src/entry.mts"
    }).to_string())).unwrap());
    let aggregate = parse_json(analyze_project_json_impl(crate::napi_api::options::test_json_arg(json!({
        "root": root, "reports": [{"type": "effects", "kind": "valkey", "entry": "src/entry.mts"}, {"type": "check"}]
    }).to_string())).unwrap());
    assert_eq!(aggregate["reports"][0]["result"], standalone);
    assert_eq!(standalone["callSites"].as_array().unwrap().len(), 3);
    assert!(aggregate["reports"][1]["result"]["rules"].as_array().unwrap().iter().any(|finding| finding["file"] == "src/entry.mts" && finding["line"] == 26));
}

#[test]
fn typed_and_returned_member_selectors_share_async_api_provenance() {
    let root = repo_fixture(&["test-cases", "rules", "query-reached-per-item", "typed-client-fixture"]);
    let standalone = parse_json(crate::napi_api::effects_json_impl(crate::napi_api::options::test_json_arg(json!({
        "root": root, "kind": "postgres", "entry": "src/entry.mts"
    }).to_string())).unwrap());
    let aggregate = parse_json(analyze_project_json_impl(crate::napi_api::options::test_json_arg(json!({
        "root": root, "reports": [{"type": "effects", "kind": "postgres", "entry": "src/entry.mts"}, {"type": "check"}]
    }).to_string())).unwrap());
    assert_eq!(aggregate["reports"][0]["result"], standalone);
    assert_eq!(standalone["callSites"].as_array().unwrap().len(), 5);
    assert!(standalone["callSites"].as_array().unwrap().iter().any(|site| site["callee"] == "writePool.connect().query"));
    assert_eq!(aggregate["reports"][1]["result"]["rules"].as_array().unwrap().len(), 16);
}
