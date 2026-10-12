#[test]
fn prepared_companion_check_matches_standalone_and_combined_reports() {
    let saved = repo_fixture(&[
        "test-cases",
        "rules",
        "required-companion-imports",
        "prepared-shared",
    ]);
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();
    let check = parse_json(
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root, "config": ".no-mistakes.yml"}),
        ))
        .unwrap(),
    );
    assert_eq!(check["rules"].as_array().unwrap().len(), 2);
    let ordinary = [
        json!({"type": "symbols", "files": ["stories/value.stories.mts", "stories/legacy.stories.js"], "include": "both"}),
        json!({"type": "dependencies", "files": ["stories/types.stories.tsx"], "relationships": ["import"]}),
    ];
    let baseline = parse_json(
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root, "reports": ordinary}),
        ))
        .unwrap(),
    );
    for reports in [
        vec![json!({"type": "check"})],
        vec![
            json!({"type": "check"}),
            ordinary[0].clone(),
            ordinary[1].clone(),
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
        if combined["reports"].as_array().unwrap().len() > 1 {
            assert_eq!(combined["reports"][1], baseline["reports"][0]);
            assert_eq!(combined["reports"][2], baseline["reports"][1]);
        } else {
            assert!(!counts.contains_key(&root.join("src/A.tsx")), "{counts:#?}");
        }
        for file in ["stories/value.stories.mts", "stories/legacy.stories.js"] {
            assert_eq!(counts.get(&root.join(file)), Some(&2), "{counts:#?}");
        }
        assert_eq!(
            counts.get(&root.join("stories/types.stories.tsx")),
            Some(&1),
            "{counts:#?}"
        );
    }
}

#[test]
fn prepared_companion_invalid_glob_preserves_aggregate_hard_error() {
    let saved = repo_fixture(&[
        "test-cases",
        "rules",
        "required-companion-imports",
        "prepared-shared",
    ]);
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();
    let config = ".invalid-glob-no-mistakes.yml";
    let standalone = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        json!({"root": root, "config": config}),
    ))
    .unwrap_err()
    .to_string();
    assert!(
        standalone.contains("error parsing glob '[': unclosed character class; missing ']'"),
        "{standalone}"
    );
    for reports in [
        vec![json!({"type": "check"})],
        vec![
            json!({"type": "check"}),
            json!({"type": "symbols", "files": ["src/A.tsx"]}),
        ],
    ] {
        let error = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root, "config": config, "reports": reports}),
        ))
        .unwrap_err()
        .to_string();
        assert_eq!(error, standalone);
    }
}

#[test]
fn prepared_companion_scope_preserves_skipped_and_explicitly_reopened_projects() {
    let saved = repo_fixture(&[
        "test-cases",
        "rules",
        "required-companion-imports",
        "prepared-scope",
    ]);
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();
    for (config, expected) in [(".no-mistakes.yml", 0), (".reopened-no-mistakes.yml", 1)] {
        let check = parse_json(
            crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root, "config": config}),
            ))
            .unwrap(),
        );
        assert_eq!(check["rules"].as_array().unwrap().len(), expected);
        for reports in [
            vec![json!({"type": "check"})],
            vec![
                json!({"type": "check"}),
                json!({"type": "symbols", "files": ["hidden/app/src/A.tsx"]}),
            ],
        ] {
            let combined = parse_json(
                analyze_project_json_impl(crate::napi_api::options::test_json_arg(
                    json!({"root": root, "config": config, "reports": reports}),
                ))
                .unwrap(),
            );
            assert_eq!(combined["reports"][0]["result"], check);
        }
    }
}
