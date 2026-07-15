#[test]
fn prepared_symbols_signature_and_flow_match_standalone_and_parse_once() {
    let source = parser_fixture("analyze-project-mixed");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let list = json!({ "root": root, "files": ["utils.mts"] });
    let impact = json!({
        "root": root,
        "files": ["utils.mts"],
        "mode": "signature-impact",
        "symbol": "parseDate"
    });
    let flow = json!({
        "root": root,
        "target": "utils.mts",
        "direction": "dependents",
        "relationships": ["import"]
    });
    let standalone = [
        parse_json(crate::napi_api::symbols_json_impl(list.to_string()).unwrap()),
        parse_json(crate::napi_api::symbols_json_impl(impact.to_string()).unwrap()),
        parse_json(crate::napi_api::flow_json_impl(flow.to_string()).unwrap()),
    ];
    crate::ast::begin_parse_count(&root);
    let output = analyze_project_json_impl(
        json!({
            "root": root,
            "reports": [
                { "type": "symbols", "files": ["utils.mts"] },
                {
                    "type": "symbols",
                    "files": ["utils.mts"],
                    "mode": "signature-impact",
                    "symbol": "parseDate"
                },
                {
                    "type": "flow",
                    "target": "utils.mts",
                    "direction": "dependents",
                    "relationships": ["import"]
                }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(report_results(&output), standalone);
    assert_eq!(counts.len(), 3, "{counts:#?}");
    assert_eq!(counts.get(&root.join("consumer.mts")), Some(&1));
    assert_eq!(counts.get(&root.join("consumer.test.mts")), Some(&1));
    assert_eq!(counts.get(&root.join("utils.mts")), Some(&2));
}

#[test]
fn mixed_check_and_symbols_keep_explicit_ignored_targets_in_the_one_parse_scope() {
    let fixture = crate::test_support::materialize_gitignore_fixture("prepared-tsconfig");
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let list = json!({
        "root": root,
        "files": ["ignored-explicit/Button.tsx"]
    });
    let impact = json!({
        "root": root,
        "files": ["ignored-explicit/Button.tsx"],
        "mode": "signature-impact",
        "symbol": "IgnoredButton"
    });
    let standalone = [
        parse_json(crate::napi_api::check_json_impl(json!({ "root": root }).to_string()).unwrap()),
        parse_json(crate::napi_api::symbols_json_impl(list.to_string()).unwrap()),
        parse_json(crate::napi_api::symbols_json_impl(impact.to_string()).unwrap()),
    ];

    crate::ast::begin_parse_count(&root);
    let output = analyze_project_json_impl(
        json!({
            "root": root,
            "reports": [
                { "type": "check" },
                { "type": "symbols", "files": ["ignored-explicit/Button.tsx"] },
                {
                    "type": "symbols",
                    "files": ["ignored-explicit/Button.tsx"],
                    "mode": "signature-impact",
                    "symbol": "IgnoredButton"
                }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(report_results(&output), standalone);
    assert_eq!(
        standalone[1]["files"][0]["path"],
        "ignored-explicit/Button.tsx"
    );
    assert!(standalone[1]["files"][0]["exports"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["name"] == "IgnoredButton"));
    assert!(standalone[2]["productionCallers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| {
            entry["file"] == "src/IgnoredButtonUser.tsx" && entry["symbol"] == "IgnoredButtonUser"
        }));

    let explicit = root.join("ignored-explicit/Button.tsx");
    let mut expected = crate::codebase::ts_source::discover_files(&root, &[])
        .into_iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
        .collect::<Vec<_>>();
    expected.push(explicit.clone());
    expected.sort();
    expected.dedup();
    assert_eq!(counts.len(), expected.len(), "{counts:#?}");
    for path in expected {
        assert_eq!(counts.get(&path), Some(&1), "{counts:#?}");
    }
    assert_eq!(counts.get(&explicit), Some(&1), "{counts:#?}");
    assert!(!counts.contains_key(&root.join("ignored-transitive/Button.test.tsx")));
}

include!("symbols/playwright.rs");
