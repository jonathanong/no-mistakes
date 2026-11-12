#[test]
fn p6_effects_and_rsc_propagate_missing_files_from_the_prepared_query() {
    let (_fixture, root) = p6_analysis_fixture("effects");
    let (_, prepared) = options(&root, json!({"type":"symbols", "files":["app/server.ts"]}));
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    let (effects, _) = options(
        &root,
        json!({
            "type":"effects", "kind":"valkey", "entry":"p6-missing.ts"
        }),
    );
    let error = scope.effects_report(&effects, &prepared).unwrap_err();
    assert!(
        error.to_string().contains("entry file not found:"),
        "{error:#}"
    );
    assert!(error.to_string().contains("p6-missing.ts"), "{error:#}");
    let (rsc, _) = options(
        &root,
        json!({
            "type":"rscCallers", "component":"p6-missing.tsx"
        }),
    );
    let error = scope.rsc_callers_report(&rsc, &prepared).unwrap_err();
    assert!(
        error.to_string().contains("component file not found:"),
        "{error:#}"
    );
    assert!(error.to_string().contains("p6-missing.tsx"), "{error:#}");
}

#[test]
fn p6_domain_reports_keep_invalid_filter_errors_on_repeated_requests() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/server-routes/canonical-relationships");
    let fixture = crate::test_support::materialize_saved_fixture(&root);
    let prepared: AnalyzeProjectOptions = serde_json::from_value(json!({
        "root":fixture.path(), "reports":[{"type":"serverRoutes"}, {"type":"queues"}]
    }))
    .unwrap();
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    for kind in ["serverRoutes", "serverRouteEdges", "queues", "queueEdges"] {
        let request: AnalyzeReportRequest = serde_json::from_value(json!({
            "type":kind, "filters":["["]
        }))
        .unwrap();
        let first = scope.project_report(&request, &prepared).unwrap_err();
        let second = scope.project_report(&request, &prepared).unwrap_err();
        assert!(format!("{first:#}").contains("glob"), "{kind}: {first:#}");
        assert_eq!(format!("{first:#}"), format!("{second:#}"));
    }
}

#[test]
fn p6_react_reports_propagate_query_and_cached_analysis_errors() {
    let (_fixture, root) = p6_analysis_fixture("simple");
    let (_, prepared) = options(&root, json!({"type":"symbols", "files":["a.mts"]}));
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    let (request, _) = options(
        &root,
        json!({
            "type":"reactUsages", "target":"p6-missing.tsx"
        }),
    );
    let error = scope.project_report(&request, &prepared).unwrap_err();
    assert!(
        error.to_string().contains("target file not found:"),
        "{error:#}"
    );
    for kind in ["reactAnalyze", "reactCheck"] {
        let (request, _) = options(&root, json!({"type":kind, "targets":["["]}));
        let first = scope.project_report(&request, &prepared).unwrap_err();
        let second = scope.project_report(&request, &prepared).unwrap_err();
        assert!(format!("{first:#}").contains("glob"), "{first:#}");
        assert_eq!(format!("{first:#}"), format!("{second:#}"));
    }
}
