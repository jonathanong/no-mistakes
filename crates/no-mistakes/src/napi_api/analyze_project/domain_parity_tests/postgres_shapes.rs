#[test]
fn prepared_sql_shape_check_matches_standalone_and_parses_builders_once() {
    let source = repo_fixture(&[
        "test-cases",
        "rules",
        "postgres-sql-shape-policy",
        "fixture",
        "prepared",
    ]);
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let standalone = parse_json(
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root, "config": ".no-mistakes.yml" }),
        ))
        .unwrap(),
    );
    crate::ast::begin_parse_count(&root);
    let aggregate = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root, "config": ".no-mistakes.yml", "reports": [{ "type": "check" }] }),
    ))
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let aggregate = parse_json(aggregate);
    assert_eq!(standalone["rules"].as_array().unwrap().len(), 2);
    assert_eq!(aggregate["reports"][0]["result"], standalone);
    assert_eq!(
        counts.get(&root.join("src/builders.ts")),
        Some(&1),
        "{counts:#?}"
    );
    assert_eq!(
        counts.get(&root.join("src/custom-executors.ts")),
        Some(&1),
        "{counts:#?}"
    );
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
}

#[test]
fn prepared_postgres_caller_projection_keys_match_standalone() {
    let root = category_fixture("rules", "postgres-bounded-statements");
    let options = json!({ "root": root, "config": "caller-projection.yml" });
    let standalone = parse_json(
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(options.clone()))
            .unwrap(),
    );
    let aggregate = parse_json(
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root,
            "config": "caller-projection.yml", "reports": [{"type": "check"}]}),
        ))
        .unwrap(),
    );
    assert_eq!(standalone["rules"].as_array().unwrap().len(), 7);
    assert_eq!(aggregate["reports"][0]["result"], standalone);
}

#[test]
fn prepared_postgres_boolean_keys_and_returning_ctes_match_standalone() {
    let root = category_fixture("rules", "postgres-bounded-statements");
    for (config, expected) in [
        ("boolean-keys.yml", 9),
        ("boolean-cte-embedded.yml", 1),
        ("cte-returning-positive.yml", 0),
        ("cte-returning-negative.yml", 8),
        ("cte-declaration-scope.yml", 1),
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
        assert_eq!(
            standalone["rules"].as_array().unwrap().len(),
            expected,
            "{config}"
        );
        assert_eq!(standalone["warnings"], json!([]), "{config}");
        assert_eq!(aggregate["reports"][0]["result"], standalone, "{config}");
    }
}

#[test]
fn prepared_postgres_unique_index_shapes_match_standalone() {
    let root = category_fixture("rules", "postgres-bounded-statements").join("unique-index-shapes");
    let options = json!({"root": root, "config": ".no-mistakes.yml"});
    let standalone = parse_json(
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(options)).unwrap(),
    );
    let aggregate = parse_json(
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(json!({
            "root": root, "config": ".no-mistakes.yml", "reports": [{"type": "check"}]
        })))
        .unwrap(),
    );
    assert_eq!(standalone["rules"].as_array().unwrap().len(), 24);
    assert_eq!(standalone["warnings"], json!([]));
    assert_eq!(aggregate["reports"][0]["result"], standalone);
}
