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

#[test]
fn symbol_dependents_index_is_seeded_once_before_parallel_reports() {
    let root = repo_fixture(&["fixtures", "performance", "core-analysis"]);
    let options: AnalyzeProjectOptions = serde_json::from_value(json!({
        "root": root,
        "tsconfig": root.join("tsconfig.json"),
        "reports": [
            {"id": "structured", "type": "dependents", "files": [{"file": "packages/core/src/index.ts", "symbol": "CoreValue"}], "relationships": ["all"]},
            {"id": "legacy", "type": "dependents", "files": ["packages/core/src/index.ts#CoreValue"], "relationships": ["all"]}
        ]
    }))
    .unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let context = crate::diagnostics::with_observer(Some(observer.clone()), || {
        context::AnalyzeProjectContext::prepare(&options).unwrap()
    });
    let prepared = observer.snapshot().work;
    assert_eq!(prepared.get("symbol_index.builds"), Some(&1));
    assert_eq!(prepared.get("traversal.computations"), Some(&1));
    let output = crate::diagnostics::with_observer(Some(observer.clone()), || {
        analyze_project_with_context(&options, &context).unwrap()
    });
    let results = report_results(&crate::cli::json_string(&output));
    assert_eq!(results[0], results[1]);
    assert!(!results[0]["files"].as_array().unwrap().is_empty());
    assert_eq!(observer.snapshot().work.get("symbol_index.builds"), Some(&1));
    assert_eq!(observer.snapshot().work.get("traversal.computations"), Some(&1));
    let ordinary: AnalyzeProjectOptions = serde_json::from_value(json!({
        "root": root,
        "reports": [{"type": "dependents", "files": ["packages/core/src/index.ts"]}]
    })).unwrap();
    let ordinary_observer = crate::diagnostics::InvocationObserver::new(true);
    let ordinary_context = crate::diagnostics::with_observer(
        Some(ordinary_observer.clone()),
        || context::AnalyzeProjectContext::prepare(&ordinary).unwrap(),
    );
    assert!(!ordinary_observer.snapshot().work.contains_key("symbol_index.builds"));
    crate::diagnostics::with_observer(Some(ordinary_observer.clone()), || {
        analyze_project_with_context(&ordinary, &ordinary_context).unwrap();
    });
    assert!(!ordinary_observer.snapshot().work.contains_key("symbol_index.builds"));
}

#[test]
fn symbol_prewarm_replays_automatic_catalog_diagnostics() {
    let root = repo_fixture(&["fixtures", "tsconfig", "workspace-resolution"]);
    let options = json!({
        "root": root,
        "reports": [
            {"type": "dependents", "files": ["apps/ambiguous/src/entry.ts#value"], "relationships": ["import"]},
            {"type": "dependents", "files": ["apps/ambiguous/src/entry.ts#value"], "relationships": ["import"]},
            {"type": "dependents", "files": ["apps/web/src/entry.ts"], "relationships": ["import"]}
        ]
    });
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(options))
        .unwrap();
    let reports = report_results(&output);
    let ambiguous = |index: usize| reports[index]["diagnostics"].as_array().unwrap().iter()
        .any(|diagnostic| diagnostic["kind"] == "ambiguous-ownership");
    assert!(ambiguous(0), "{output}");
    assert_eq!(reports[0], reports[1]);
    assert!(!ambiguous(2), "{output}");
}
