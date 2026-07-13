#[test]
fn prepared_check_matches_standalone_and_parses_its_scope_once() {
    let source = analysis_fixture("unique-exports-basic");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let options = json!({
        "root": root,
        "config": ".no-mistakes.yml",
        "tsconfig": "tsconfig.json"
    });
    let standalone = parse_json(crate::napi_api::check_json_impl(options.to_string()).unwrap());

    crate::ast::begin_parse_count(&root);
    let aggregate = analyze_project_json_impl(
        json!({
            "root": root,
            "config": ".no-mistakes.yml",
            "tsconfig": "tsconfig.json",
            "reports": [{ "type": "check", "id": "check" }]
        })
        .to_string(),
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    let aggregate = parse_json(aggregate);

    assert_eq!(aggregate["reports"][0]["result"], standalone);
    assert_eq!(
        counts.len(),
        2,
        "check scope must parse only its two TS inputs: {counts:#?}"
    );
    assert_eq!(counts.get(&root.join("src/a.ts")), Some(&1), "{counts:#?}");
    assert_eq!(counts.get(&root.join("src/b.ts")), Some(&1), "{counts:#?}");
}

#[test]
fn check_only_report_follows_reachable_dynamic_imports_like_standalone_check() {
    let root = repo_fixture(&[
        "fixtures",
        "napi",
        "analyze-project-dynamic-import-reachability",
    ]);
    let standalone =
        parse_json(crate::napi_api::check_json_impl(json!({ "root": root }).to_string()).unwrap());

    let aggregate = parse_json(
        analyze_project_json_impl(
            json!({
                "root": root,
                "reports": [{ "type": "check" }]
            })
            .to_string(),
        )
        .unwrap(),
    );
    let aggregate_check = &aggregate["reports"][0]["result"];

    assert_eq!(aggregate_check, &standalone);
    assert!(aggregate_check["rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| {
            finding["file"] == "src/reachable.ts" && finding["target"] == "src/lazy.ts"
        }));
}

#[test]
fn prepared_effects_and_rsc_reports_match_standalone_outputs() {
    let effects_root = parser_fixture("effects");
    let rsc_root = parser_fixture("rsc");
    let effects_options = json!({
        "root": effects_root,
        "kind": "storage",
        "entry": "entry.ts"
    });
    let rsc_options = json!({
        "root": rsc_root,
        "component": "app/Target.tsx"
    });
    let standalone = [
        parse_json(crate::napi_api::effects_json_impl(effects_options.to_string()).unwrap()),
        parse_json(crate::napi_api::rsc_callers_json_impl(rsc_options.to_string()).unwrap()),
    ];

    let output = analyze_project_json_impl(
        json!({
            "root": effects_root,
            "reports": [
                { "type": "effects", "kind": "storage", "entry": "entry.ts" },
                { "type": "rscCallers", "root": rsc_root, "component": "app/Target.tsx" }
            ]
        })
        .to_string(),
    )
    .unwrap();

    assert_eq!(report_results(&output), standalone);
}

#[test]
fn prepared_effects_and_rsc_keep_explicit_ignored_roots_authoritative() {
    let fixture = crate::test_support::materialize_gitignore_fixture("prepared-tsconfig");
    let root = fixture.path();
    let standalone = [
        parse_json(
            crate::napi_api::effects_json_impl(
                json!({
                    "root": root,
                    "kind": "regression",
                    "entry": "ignored-explicit/effect-entry.ts"
                })
                .to_string(),
            )
            .unwrap(),
        ),
        parse_json(
            crate::napi_api::rsc_callers_json_impl(
                json!({
                    "root": root,
                    "component": "ignored-explicit/Button.tsx"
                })
                .to_string(),
            )
            .unwrap(),
        ),
    ];

    let output = analyze_project_json_impl(
        json!({
            "root": root,
            "reports": [
                {
                    "type": "effects",
                    "kind": "regression",
                    "entry": "ignored-explicit/effect-entry.ts"
                },
                {
                    "type": "rscCallers",
                    "component": "ignored-explicit/Button.tsx"
                }
            ]
        })
        .to_string(),
    )
    .unwrap();

    assert_eq!(report_results(&output), standalone);
}
