#[test]
fn sql_verdict_reports_have_independent_opt_in_sinks() {
    let root = repo_fixture(&[
        "test-cases",
        "rules",
        "postgres-require-query-annotation",
        "fixture",
        "provenance-advice",
    ]);
    let aggregate = parse_json(
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(json!({
            "root": root,
            "config": ".no-mistakes.yml",
            "reports": [
                {"type": "check", "includeSqlVerdicts": true},
                {"type": "check", "includeSqlVerdicts": false},
                {"type": "check", "includeSqlVerdicts": true}
            ]
        })))
        .unwrap(),
    );
    let first = &aggregate["reports"][0]["result"];
    let middle = &aggregate["reports"][1]["result"];
    let last = &aggregate["reports"][2]["result"];
    assert_eq!(first, last, "{aggregate}");
    assert_eq!(first["sqlVerdicts"].as_array().unwrap().len(), 5);
    assert!(middle.get("sqlVerdicts").is_none(), "{aggregate}");
    let mut without_verdicts = first.clone();
    without_verdicts
        .as_object_mut()
        .unwrap()
        .remove("sqlVerdicts");
    assert_eq!(&without_verdicts, middle);
    let standalone = parse_json(
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(json!({
            "root": root,
            "config": ".no-mistakes.yml",
            "includeSqlVerdicts": true
        })))
        .unwrap(),
    );
    assert_eq!(*first, standalone);
}
