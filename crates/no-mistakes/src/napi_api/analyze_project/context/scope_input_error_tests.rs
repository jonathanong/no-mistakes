use super::*;

#[test]
fn prepared_report_helpers_reject_bad_request_types_at_their_input_boundary() {
    let prepared = options(json!([{"type":"symbols", "files":["a.mts"]}]));
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    let mut failures = Vec::new();
    for report in [
        json!({"type":"dependencies", "files":["a.mts"], "depth":"deep"}),
        json!({"type":"importUsages", "filters":false}),
        json!({"type":"symbols", "files":42}),
        json!({"type":"flow", "target":42}),
        json!({"type":"effects", "kind":"fetch", "entry":42}),
        json!({"type":"rscCallers", "component":false}),
        json!({"type":"callSites", "file":42, "exportName":"a"}),
    ] {
        let request: AnalyzeReportRequest = serde_json::from_value(report).unwrap();
        let result = match request.report_type.as_str() {
            "dependencies" => scope.graph_report(&request, &prepared, Direction::Deps),
            "importUsages" => scope.import_usages_report(&request, &prepared),
            "symbols" => scope.symbols_report(&request, &prepared),
            "flow" => scope.flow_report(&request, &prepared),
            "effects" => scope.effects_report(&request, &prepared),
            "rscCallers" => scope.rsc_callers_report(&request, &prepared),
            "callSites" => scope.call_sites_report(&request, &prepared),
            _ => unreachable!(),
        };
        match result {
            Err(error) if format!("{error:#}").contains("invalid type") => {}
            Err(error) => failures.push(format!(
                "{}: wrong boundary error {error:#}",
                request.report_type
            )),
            Ok(_) => failures.push(format!(
                "{} accepted a malformed request",
                request.report_type
            )),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn symbols_report_rejects_empty_files_and_unsupported_selection_values() {
    let prepared = options(json!([{"type":"symbols", "files":["a.mts"]}]));
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let scope = context.scopes.values().next().unwrap();
    for (report, expected) in [
        (json!({"type":"symbols", "files":[]}), "files"),
        (
            json!({"type":"symbols", "files":["a.mts"], "mode":"missing-mode"}),
            "mode",
        ),
        (
            json!({"type":"symbols", "files":["a.mts"], "kinds":["missing-kind"]}),
            "kind",
        ),
        (
            json!({"type":"symbols", "files":["a.mts"], "include":"missing-include"}),
            "include",
        ),
    ] {
        let request = serde_json::from_value(report).unwrap();
        let error = scope.symbols_report(&request, &prepared).unwrap_err();
        assert!(format!("{error:#}").contains(expected), "{error:#}");
    }
}
