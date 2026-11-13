fn p6_analysis_fixture(name: &str) -> (tempfile::TempDir, String) {
    let fixture =
        crate::test_support::materialize_saved_fixture(&PathBuf::from(fixture_root(name)));
    let root = fixture.path().canonicalize().unwrap().display().to_string();
    (fixture, root)
}

#[test]
fn p6_call_sites_requires_a_prepared_projection() {
    let (_fixture, root) = p6_analysis_fixture("simple");
    let (_, prepared) = options(&root, json!({"type":"symbols", "files":["a.mts"]}));
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    let (call, _) = options(
        &root,
        json!({"type":"callSites", "file":"a.mts", "exportName":"a"}),
    );
    let error = scope.call_sites_report(&call, &prepared).unwrap_err();
    assert_eq!(
        error.to_string(),
        "ordinary call-sites projection is missing"
    );
}

#[test]
fn p6_call_sites_rejects_a_fatal_target_even_without_a_projection() {
    let (_fixture, root) = p6_analysis_fixture("forbidden-dependencies-parse-error");
    let (_, prepared) = options(
        &root,
        json!({"type":"symbols", "files":["entrypoints/broken.mts"]}),
    );
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    let (request, _) = options(
        &root,
        json!({"type":"callSites", "file":"entrypoints/broken.mts", "exportName":"broken"}),
    );
    let error = scope.call_sites_report(&request, &prepared).unwrap_err();
    assert!(
        error.to_string().contains("fatal parser failure"),
        "{error:#}"
    );
    assert!(error.to_string().contains("broken.mts"), "{error:#}");
}

#[test]
fn p6_project_and_playwright_reports_require_their_prepared_inputs() {
    let (_fixture, root) = p6_analysis_fixture("simple");
    let (_, prepared) = options(&root, json!({"type":"symbols", "files":["a.mts"]}));
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    for (report, expected) in [
        (
            json!({"type":"serverRoutes"}),
            "server analysis was not prepared",
        ),
        (json!({"type":"check"}), "check analysis was not prepared"),
    ] {
        let request = serde_json::from_value(report).unwrap();
        let error = scope.project_report(&request, &prepared).unwrap_err();
        assert_eq!(error.to_string(), expected);
    }
    let (request, _) = options(&root, json!({"type":"playwrightCheck"}));
    let error = scope.playwright_report(&request, &prepared).unwrap_err();
    assert_eq!(
        error.to_string(),
        "distinct Playwright settings require a separate prepared analyzeProject context"
    );
}

#[test]
fn p6_graph_report_validates_candidate_direction_at_render_time() {
    let (_fixture, root) = p6_analysis_fixture("simple");
    let (_, prepared) = options(&root, json!({"type":"symbols", "files":["a.mts"]}));
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    let (request, _) = options(
        &root,
        json!({
            "type":"dependents", "files":["a.mts"],
            "candidateInclude":["*.mts"], "relationships":["import"]
        }),
    );
    let error = scope
        .graph_report(&request, &prepared, Direction::Dependents)
        .unwrap_err();
    assert!(
        error.to_string().contains("only valid for dependencies"),
        "{error:#}"
    );
}
