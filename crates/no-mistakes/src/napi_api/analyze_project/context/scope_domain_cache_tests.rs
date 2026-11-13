#[test]
fn p6_queue_reports_reuse_plain_and_indexed_analysis_without_crossing_caches() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/basic/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&root);
    let root = fixture.path().canonicalize().unwrap();
    let prepared: AnalyzeProjectOptions = serde_json::from_value(json!({
        "root":root, "reports":[{"type":"queues"}]
    }))
    .unwrap();
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    let first = scope
        .project_report(&prepared.reports[0], &prepared)
        .unwrap();
    let first: Value = serde_json::from_str(first.get()).unwrap();
    assert!(
        !first["producers"].as_array().unwrap().is_empty(),
        "{first}"
    );
    let edges: AnalyzeReportRequest = serde_json::from_value(json!({
        "type":"queueEdges", "files":["enqueue.ts"], "depth":2
    }))
    .unwrap();
    let edges = scope.project_report(&edges, &prepared).unwrap();
    let edges: Value = serde_json::from_str(edges.get()).unwrap();
    assert!(!edges.as_array().unwrap().is_empty(), "{edges}");
    let repeated = scope
        .project_report(&prepared.reports[0], &prepared)
        .unwrap();
    assert_eq!(
        first,
        serde_json::from_str::<Value>(repeated.get()).unwrap()
    );
    assert_eq!(scope.queue_reports.lock().unwrap().len(), 1);
    assert_eq!(scope.queue_indexed_reports.lock().unwrap().len(), 1);
}

#[test]
fn p6_server_reports_reuse_plain_and_indexed_analysis_without_crossing_caches() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/server-routes/canonical-relationships");
    let fixture = crate::test_support::materialize_saved_fixture(&root);
    let root = fixture.path().canonicalize().unwrap();
    let prepared: AnalyzeProjectOptions = serde_json::from_value(json!({
        "root":root, "reports":[{"type":"serverRoutes"}]
    }))
    .unwrap();
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    let first = scope
        .project_report(&prepared.reports[0], &prepared)
        .unwrap();
    let first: Value = serde_json::from_str(first.get()).unwrap();
    assert!(!first["routes"].as_array().unwrap().is_empty(), "{first}");
    assert!(
        first["routes"].as_array().unwrap().iter().any(|route| {
            route["file"] == "backend/api/users.ts"
                && route["route"] == "/api/v1/users/*"
                && route["method"] == "get"
        }),
        "{first}"
    );
    let edges: AnalyzeReportRequest = serde_json::from_value(json!({
        "type":"serverRouteEdges", "files":["backend/api/users.ts"]
    }))
    .unwrap();
    let edges = scope.project_report(&edges, &prepared).unwrap();
    let edges: Value = serde_json::from_str(edges.get()).unwrap();
    assert!(!edges.as_array().unwrap().is_empty(), "{edges}");
    assert!(
        edges.as_array().unwrap().contains(&json!({
            "from":"backend/api/users.ts", "to":"/api/v1/users/*", "kind":"server-route"
        })),
        "{edges}"
    );
    assert!(
        edges.as_array().unwrap().iter().all(|edge| {
            edge["from"] == "backend/api/users.ts" && edge["kind"] == "server-route"
        }),
        "{edges}"
    );
    let repeated = scope
        .project_report(&prepared.reports[0], &prepared)
        .unwrap();
    assert_eq!(
        first,
        serde_json::from_str::<Value>(repeated.get()).unwrap()
    );
    assert_eq!(scope.server_reports.lock().unwrap().len(), 1);
    assert_eq!(scope.server_indexed_reports.lock().unwrap().len(), 1);
}

#[test]
fn p6_react_usages_reject_missing_target_and_invalid_include_before_analysis() {
    let (_fixture, root) = p6_analysis_fixture("simple");
    let (_, prepared) = options(&root, json!({"type":"symbols", "files":["a.mts"]}));
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    for (report, expected) in [
        (
            json!({"type":"reactUsages"}),
            "target is required for react usages",
        ),
        (
            json!({"type":"reactUsages", "target":"a.mts", "include":"bogus"}),
            "unknown --include section: bogus (expected stories, tests, props)",
        ),
    ] {
        let request = serde_json::from_value(report).unwrap();
        let error = scope.project_report(&request, &prepared).unwrap_err();
        assert_eq!(error.to_string(), expected);
    }
    assert!(scope.react_analyses.lock().unwrap().is_empty());
}
