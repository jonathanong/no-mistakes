#[test]
fn prepared_sql_shape_check_matches_standalone_and_parses_builders_once() {
    let source = repo_fixture(&["test-cases", "rules", "postgres-sql-shape-policy", "fixture", "prepared"]);
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let standalone = parse_json(crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(json!({ "root": root, "config": ".no-mistakes.yml" }))).unwrap());
    crate::ast::begin_parse_count(&root);
    let aggregate = analyze_project_json_impl(crate::napi_api::options::test_json_arg(json!({ "root": root, "config": ".no-mistakes.yml", "reports": [{ "type": "check" }] }))).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let aggregate = parse_json(aggregate);
    assert_eq!(standalone["rules"].as_array().unwrap().len(), 2);
    assert_eq!(aggregate["reports"][0]["result"], standalone);
    assert_eq!(counts.get(&root.join("src/builders.ts")), Some(&1), "{counts:#?}");
    assert_eq!(counts.len(), 1, "{counts:#?}");
}
