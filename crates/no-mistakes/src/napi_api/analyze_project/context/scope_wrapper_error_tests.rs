type P6ReportFn = fn(
    &AnalyzeProjectContext,
    &AnalyzeReportRequest,
    &AnalyzeProjectOptions,
) -> anyhow::Result<Box<serde_json::value::RawValue>>;

fn p6_graph_wrapper(
    context: &AnalyzeProjectContext,
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
) -> anyhow::Result<Box<serde_json::value::RawValue>> {
    context.graph_report(request, options, Direction::Deps)
}

#[test]
fn p6_report_wrappers_propagate_invalid_and_unprepared_scope_errors() {
    let (_fixture, root) = p6_analysis_fixture("simple");
    let (_, prepared) = options(&root, json!({"type":"symbols", "files":["a.mts"]}));
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let wrappers: [(&str, P6ReportFn); 10] = [
        ("dependencies", p6_graph_wrapper),
        ("importUsages", AnalyzeProjectContext::import_usages_report),
        ("symbols", AnalyzeProjectContext::symbols_report),
        ("flow", AnalyzeProjectContext::flow_report),
        ("effects", AnalyzeProjectContext::effects_report),
        ("rscCallers", AnalyzeProjectContext::rsc_callers_report),
        ("queues", AnalyzeProjectContext::project_report),
        ("playwrightCheck", AnalyzeProjectContext::playwright_report),
        ("callSites", AnalyzeProjectContext::command_report),
        (
            "resolveCheckDependencies",
            AnalyzeProjectContext::resolve_check_dependencies_report,
        ),
    ];
    for (kind, wrapper) in wrappers {
        let request = serde_json::from_value(json!({"type":kind, "root":false})).unwrap();
        let error = wrapper(&context, &request, &prepared).unwrap_err();
        assert_eq!(error.to_string(), "root must be a string", "{kind}");
        let request = serde_json::from_value(json!({
            "type":kind, "config":"p6-unprepared.yml"
        }))
        .unwrap();
        let error = wrapper(&context, &request, &prepared).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("prepared analyzeProject scope is missing"),
            "{kind}: {error:#}"
        );
        assert!(
            error.to_string().contains("p6-unprepared.yml"),
            "{kind}: {error:#}"
        );
    }
}

#[test]
fn p6_project_report_input_errors_survive_the_scope_boundary() {
    let (_fixture, root) = p6_analysis_fixture("simple");
    let (_, prepared) = options(&root, json!({"type":"symbols", "files":["a.mts"]}));
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    let project: AnalyzeReportRequest = serde_json::from_value(json!({
        "type":"queues", "files":false
    }))
    .unwrap();
    let error = scope.project_report(&project, &prepared).unwrap_err();
    assert!(error.to_string().contains("invalid type"), "{error:#}");
    let playwright: AnalyzeReportRequest = serde_json::from_value(json!({
        "type":"playwrightCheck", "project":false
    }))
    .unwrap();
    let error = scope.playwright_report(&playwright, &prepared).unwrap_err();
    assert!(error.to_string().contains("invalid type"), "{error:#}");
    let command: AnalyzeReportRequest = serde_json::from_value(json!({
        "type":"callSites", "file":"a.mts", "exportName":"a", "tsconfig":false
    }))
    .unwrap();
    let error = scope.call_sites_report(&command, &prepared).unwrap_err();
    assert!(error.to_string().contains("invalid type"), "{error:#}");
}
