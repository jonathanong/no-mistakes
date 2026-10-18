#[test]
fn shared_resolver_visibility_keeps_additive_test_impact_fields_stable() {
    let root = repo_fixture(&["fixtures", "importers", "tests-scoped-alias"]);
    let file = "packages/unit/src/subject.ts";
    let query = |tests| {
        parse_json(
            crate::napi_api::queries::importers_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root, "file": file, "tests": tests}),
            ))
            .unwrap(),
        )
    };
    let baseline = query(false);
    let with_tests = query(true);
    // A broader runner alias catalog must never answer the ordinary fields.
    assert_eq!(baseline["directImporters"], json!([]));
    assert_eq!(baseline["dependentsCount"], 0);
    assert_eq!(with_tests["directImporters"], baseline["directImporters"]);
    assert_eq!(with_tests["dependentsCount"], baseline["dependentsCount"]);
    assert_eq!(
        with_tests["testImpact"]["tests"],
        json!(["packages/unit/tests/owner.test.ts"])
    );
}

#[test]
fn multi_report_shares_resolver_keys_without_losing_known_dependencies() {
    let root = repo_fixture(&["fixtures", "performance", "core-analysis"]);
    let options = json!({
        "root": root,
        "tsconfig": root.join("tsconfig.json"),
        "reports": [
            {"id": "dependencies", "type": "dependencies", "files": ["src/app.tsx"], "relationships": ["all"]},
            {"id": "dependents", "type": "dependents", "files": ["packages/core/src/index.ts"]},
            {"id": "symbol-dependents", "type": "dependents", "files": [{"file": "packages/core/src/index.ts", "symbol": "CoreValue"}], "relationships": ["all"]},
            {"id": "symbols", "type": "symbols", "files": ["src/app.tsx"], "include": "both"}
        ]
    });
    let run = || {
        crate::ast::with_request_parse_cache(|| {
            analyze_project_json_impl(crate::napi_api::options::test_json_arg(options.to_string()))
        })
        .unwrap()
    };
    let plain = run();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let observed = crate::diagnostics::with_observer(Some(observer.clone()), run);
    assert_eq!(observed, plain);
    let reports = report_results(&observed);
    assert_eq!(reports.len(), 4);
    // Equality alone could preserve an empty/Unknown result after lost resolution.
    assert_eq!(
        reports[0]["files"],
        json!([
            {"path": "packages/core/src/index.ts", "depth": 1, "via": ["import"]},
            {"path": "src/button.tsx", "depth": 1, "via": ["import", "react-render"]}
        ])
    );
    let work = observer.snapshot().work;
    assert_eq!(work["graph.builds"], 1);
    assert!(work["graph.reuses"] >= 1);
    assert_eq!(work["symbol_index.builds"], 1);
    assert_eq!(work["resolver.computations"], 10);
    assert_eq!(work["resolver.unique_keys"], 10);
    assert!(work["resolver.computations"] < work["resolver.requests"]);
}
