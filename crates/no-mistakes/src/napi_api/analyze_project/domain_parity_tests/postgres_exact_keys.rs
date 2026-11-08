#[test]
fn prepared_postgres_exact_key_proofs_match_standalone() {
    let root = repo_fixture(&[
        "test-cases",
        "rules",
        "postgres-bounded-statements",
        "issue-3114-exact",
        "fixture",
    ]);
    for (config, expected) in [
        ("r1.yml", 1),
        ("r2.yml", 2),
        ("r3.yml", 4),
        ("r4.yml", 2),
        ("r5.yml", 6),
        ("off.yml", 0),
        ("off-sql.yml", 1),
    ] {
        let standalone = parse_json(
            crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root, "config": config}),
            ))
            .unwrap(),
        );
        let aggregate = parse_json(
            analyze_project_json_impl(crate::napi_api::options::test_json_arg(json!({
                "root": root, "config": config, "reports": [{"type": "check"}]
            })))
            .unwrap(),
        );
        assert_eq!(standalone["rules"].as_array().unwrap().len(), expected, "{config}");
        assert_eq!(standalone["warnings"], json!([]), "{config}");
        assert_eq!(aggregate["reports"][0]["result"], standalone, "{config}");
    }
}
