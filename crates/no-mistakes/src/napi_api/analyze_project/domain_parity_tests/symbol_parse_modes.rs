#[test]
fn mixed_check_and_legacy_symbols_parse_each_required_semantic_mode_once() {
    let root = repo_fixture(&[
        "fixtures",
        "napi",
        "analyze-project-mixed-symbol-parse-modes",
    ]);
    let files = ["types.js", "types.mjs"];
    let standalone = [
        parse_json(
            crate::napi_api::check_json_impl(
                json!({ "root": root, "config": ".no-mistakes.yml" }).to_string(),
            )
            .unwrap(),
        ),
        parse_json(
            crate::napi_api::symbols_json_impl(
                json!({ "root": root, "files": files }).to_string(),
            )
            .unwrap(),
        ),
        parse_json(
            crate::napi_api::flow_json_impl(
                json!({
                    "root": root,
                    "target": "types.js",
                    "direction": "deps",
                    "relationships": ["import"]
                })
                .to_string(),
            )
            .unwrap(),
        ),
    ];

    crate::ast::begin_parse_count(&root);
    let output = analyze_project_json_impl(
        json!({
            "root": root,
            "config": ".no-mistakes.yml",
            "reports": [
                { "type": "check" },
                { "type": "symbols", "files": files },
                {
                    "type": "flow",
                    "target": "types.js",
                    "direction": "deps",
                    "relationships": ["import"]
                }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(report_results(&output), standalone);
    // OXC must parse these extensions as JavaScript for check diagnostics and
    // as TypeScript for the legacy list-symbols contract.
    assert_eq!(counts.len(), files.len(), "{counts:#?}");
    for file in files {
        assert_eq!(counts.get(&root.join(file)), Some(&2), "{counts:#?}");
    }
}

#[test]
fn compatible_legacy_symbol_cache_hit_retains_fatal_panic_semantics() {
    let root = repo_fixture(&[
        "fixtures",
        "napi",
        "analyze-project-legacy-symbol-panic",
    ]);
    let request = json!({ "root": root, "files": ["invalid.ts"] });
    let standalone = crate::napi_api::symbols_json_impl(request.to_string()).unwrap_err();

    crate::ast::begin_parse_count(&root);
    let aggregate = analyze_project_json_impl(
        json!({
            "root": root,
            "reports": [{ "type": "symbols", "files": ["invalid.ts"] }]
        })
        .to_string(),
    )
    .unwrap_err();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(aggregate.reason, standalone.reason);
    assert_eq!(counts.get(&root.join("invalid.ts")), Some(&1));
}
