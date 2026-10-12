#[test]
fn prepared_jsx_check_matches_ordinary_and_aggregate_reports_with_one_physical_parse() {
    let saved = repo_fixture(&[
        "test-cases",
        "rules",
        "test-no-dependency-pins",
        "prepared-jsx",
    ]);
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();
    crate::ast::begin_parse_count(&root);
    let check = parse_json(
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root, "config": ".no-mistakes.yml"}),
        ))
        .unwrap(),
    );
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.len(), 4, "{counts:#?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
    assert_eq!(check["rules"].as_array().unwrap().len(), 4, "{check:#}");
    let ordinary = json!({"type": "symbols", "files": ["src/pins.test.tsx", "src/helper.tsx"]});
    let baseline = parse_json(
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root, "reports": [ordinary.clone()]}),
        ))
        .unwrap(),
    );
    for reports in [
        vec![json!({"type": "check"})],
        vec![
            json!({"type": "check"}),
            json!({"type": "symbols", "files": ["src/pins.test.tsx", "src/helper.tsx"]}),
        ],
    ] {
        crate::ast::begin_parse_count(&root);
        let combined = parse_json(
            analyze_project_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root, "config": ".no-mistakes.yml", "reports": reports}),
            ))
            .unwrap(),
        );
        let counts = crate::ast::finish_parse_count(&root);
        assert_eq!(combined["reports"][0]["result"], check);
        assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
        assert_eq!(counts.get(&root.join("src/pins.test.jsx")), Some(&1));
        if combined["reports"].as_array().unwrap().len() == 1 {
            assert!(!counts.contains_key(&root.join("src/helper.tsx")));
            assert!(!counts.contains_key(&root.join("src/excluded.test.tsx")));
        } else {
            assert_eq!(combined["reports"][1], baseline["reports"][0]);
        }
    }
}

#[test]
fn prepared_jsx_custom_and_invalid_patterns_preserve_aggregate_behavior() {
    let saved = repo_fixture(&[
        "test-cases",
        "rules",
        "test-no-dependency-pins",
        "prepared-jsx",
    ]);
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();
    for config in [
        ".custom-no-mistakes.yml",
        ".invalid-filter-no-mistakes.yml",
        ".invalid-pattern-no-mistakes.yml",
        ".invalid-regex-no-mistakes.yml",
    ] {
        let standalone = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root, "config": config}),
        ));
        crate::ast::begin_parse_count(&root);
        let aggregate = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root, "config": config, "reports": [{"type": "check"}]}),
        ));
        assert!(crate::ast::finish_parse_count(&root).is_empty());
        match (standalone, aggregate) {
            (Ok(standalone), Ok(aggregate)) => assert_eq!(
                parse_json(aggregate)["reports"][0]["result"],
                parse_json(standalone)
            ),
            (Err(standalone), Err(aggregate)) => {
                assert_eq!(standalone.to_string(), aggregate.to_string())
            }
            results => panic!("mismatched check results: {results:?}"),
        }
    }
}
