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
    // Registry of violating physical statements, independently matching the
    // saved catalog's expression, partial-predicate, and null-safety cases.
    let cases: &[(&str, &[u64])] = &[
        ("expression-unique.sql", &[4, 5, 8]),
        ("index-guards.sql", &[2, 4, 6]),
        ("joins-and-conjunctions.sql", &[3, 5, 7, 10, 11, 13]),
        ("nulls-not-distinct.sql", &[3]),
        ("nullsafe-expressions.sql", &[3]),
        ("partial-column-nullguard.sql", &[3]),
        ("partial-expression-unique.sql", &[2, 4, 5, 6]),
        ("partial-unique.sql", &[3, 4, 5, 6, 7]),
    ];
    let mut expected: Vec<_> = cases
        .iter()
        .flat_map(|(file, lines)| lines.iter().map(move |line| (*file, *line)))
        .collect();
    let mut actual: Vec<_> = standalone["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            assert_eq!(
                row["rule"],
                crate::codebase::rules::postgres_bounded_statements::RULE_ID
            );
            (row["file"].as_str().unwrap(), row["line"].as_u64().unwrap())
        })
        .collect();
    expected.sort_unstable();
    actual.sort_unstable();
    assert_eq!(actual, expected);
    assert_eq!(standalone["warnings"], json!([]));
    assert_eq!(aggregate["reports"][0]["result"], standalone);
}

#[test]
fn prepared_postgres_catalog_bounds_match_standalone() {
    let fixture = category_fixture("rules", "postgres-bounded-statements");
    for (directory, config, expected) in [
        ("catalog-resolution", "key-resolution.yml", 6),
        ("catalog-resolution", "key-operators.yml", 4),
        ("catalog-resolution", "key-booleans.yml", 1),
        ("catalog-resolution", "view-bounds.yml", 1),
        ("catalog-resolution", "view-allow.yml", 0),
        ("catalog-resolution", "view-suppression.yml", 0),
        ("catalog-resolution-identity", "view-identity.yml", 1),
        ("schemaless-key-resolution", "qualified-bare-tables.yml", 4),
    ] {
        let root = fixture.join(directory);
        let standalone = parse_json(
            crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root, "config": config, "includeSqlVerdicts": true}),
            ))
            .unwrap(),
        );
        let aggregate = parse_json(
            analyze_project_json_impl(crate::napi_api::options::test_json_arg(json!({
                "root": root, "config": config,
                "reports": [{"type": "check", "includeSqlVerdicts": true}]
            })))
            .unwrap(),
        );
        assert_eq!(
            standalone["rules"].as_array().unwrap().len(),
            expected,
            "{config}"
        );
        assert_eq!(aggregate["reports"][0]["result"], standalone, "{config}");
    }
}

#[test]
fn prepared_postgres_joined_dml_and_catalog_guards_match_standalone() {
    let root = category_fixture("rules", "postgres-bounded-statements").join("joined-dml");
    let standalone = parse_json(
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(json!({
            "root": root, "config": ".no-mistakes.yml"
        })))
        .unwrap(),
    );
    let aggregate = parse_json(
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(json!({
            "root": root, "config": ".no-mistakes.yml", "reports": [{"type": "check"}]
        })))
        .unwrap(),
    );
    assert_eq!(standalone["rules"].as_array().unwrap().len(), 6);
    assert_eq!(standalone["warnings"], json!([]));
    assert_eq!(aggregate["reports"][0]["result"], standalone);
}
