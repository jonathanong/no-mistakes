#[test]
fn production_check_matches_standalone_and_preserves_combined_symbol_and_import_reports() {
    let saved = repo_fixture(&[
        "test-cases",
        "rules",
        "production-dependency-declarations",
        "prepared-projection-parity",
    ]);
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();
    let check = parse_json(
        crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root, "config": ".no-mistakes.yml"}),
        ))
        .unwrap(),
    );
    assert_eq!(check["rules"].as_array().unwrap().len(), 9);
    let ordinary = [
        json!({"type": "symbols", "files": ["packages/lib/index.mts"], "include": "both"}),
        json!({"type": "dependencies", "files": ["packages/lib/index.mts"], "relationships": ["import"]}),
    ];
    let baseline = parse_json(
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root, "reports": ordinary}),
        ))
        .unwrap(),
    );
    let alone = parse_json(
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root, "config": ".no-mistakes.yml", "reports": [{"type": "check"}]}),
        ))
        .unwrap(),
    );
    assert_eq!(alone["reports"][0]["result"], check);
    crate::ast::begin_parse_count(&root);
    let combined = parse_json(
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root, "config": ".no-mistakes.yml", "reports": [
                {"type": "check"}, ordinary[0], ordinary[1],
            ]}),
        ))
        .unwrap(),
    );
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(combined["reports"][0]["result"], check);
    assert_eq!(combined["reports"][1], baseline["reports"][0]);
    assert_eq!(combined["reports"][2], baseline["reports"][1]);
    // The symbol report and production projection share the same legacy parse.
    assert_eq!(
        counts.get(&root.join("packages/lib/index.mts")),
        Some(&2),
        "{counts:#?}"
    );
    assert_eq!(
        counts.get(&root.join("packages/lib/plain.md")),
        Some(&1),
        "{counts:#?}"
    );
    assert_eq!(
        counts.get(&root.join("packages/app/index.ts")),
        Some(&1),
        "{counts:#?}"
    );
}

#[test]
fn production_check_preserves_runner_helper_import_projections_in_batched_reports() {
    let saved = repo_fixture(&[
        "test-cases",
        "rules",
        "production-dependency-declarations",
        "prepared-runner-projection-parity",
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
    let symbol_request = json!({"type": "symbols", "files": ["packages/app/runner-helper.mts", "packages/app/typed-runner.js"], "include": "both"});
    let baseline = parse_json(
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root, "reports": [symbol_request]}),
        ))
        .unwrap(),
    );
    for reports in [
        vec![json!({"type": "check"})],
        vec![json!({"type": "check"}), symbol_request],
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
        }
        for file in [
            "packages/app/runner-helper.mts",
            "packages/app/typed-runner.js",
        ] {
            assert_eq!(counts.get(&root.join(file)), Some(&2), "{counts:#?}");
        }
    }
}
