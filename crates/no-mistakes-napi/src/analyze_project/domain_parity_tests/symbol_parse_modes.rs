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
            crate::check_json_impl(
                crate::options::test_json_arg(json!({ "root": root, "config": ".no-mistakes.yml" }).to_string(),)
            )
            .unwrap(),
        ),
        parse_json(
            crate::symbols_json_impl(
                crate::options::test_json_arg(json!({ "root": root, "files": files }).to_string(),)
            )
            .unwrap(),
        ),
        parse_json(
            crate::flow_json_impl(
                crate::options::test_json_arg(json!({
                    "root": root,
                    "target": "types.js",
                    "direction": "deps",
                    "relationships": ["import"]
                })
                .to_string(),)
            )
            .unwrap(),
        ),
    ];

    no_mistakes::ast::begin_parse_count(&root);
    let output = analyze_project_json_impl(
        crate::options::test_json_arg(json!({
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
        .to_string(),)
    )
    .unwrap();
    let counts = no_mistakes::ast::finish_parse_count(&root);

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
    let standalone = crate::symbols_json_impl(crate::options::test_json_arg(request)).unwrap_err();

    no_mistakes::ast::begin_parse_count(&root);
    let aggregate = analyze_project_json_impl(
        crate::options::test_json_arg(json!({
            "root": root,
            "reports": [{ "type": "symbols", "files": ["invalid.ts"] }]
        })
        .to_string(),)
    )
    .unwrap_err();
    let counts = no_mistakes::ast::finish_parse_count(&root);

    assert_eq!(aggregate.reason, standalone.reason);
    assert_eq!(counts.get(&root.join("invalid.ts")), Some(&1));
}
