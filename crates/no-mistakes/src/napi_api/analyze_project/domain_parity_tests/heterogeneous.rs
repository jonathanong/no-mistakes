#[test]
fn heterogeneous_request_keeps_compatible_and_check_scopes_deterministic() {
    // Use fixtures distinct from parser-count tests so concurrent test execution cannot add
    // unrelated parses to another test's active session.
    let queue_root = analysis_fixture("queue-dashboard/good");
    let playwright_root = category_fixture("nextjs-coverage", "covered");
    let react_root = category_fixture("react-traits-analyze", "multi-component");
    let server_root = analysis_fixture("routes/good");
    let effects_root = analysis_fixture("effects");
    let rsc_root = analysis_fixture("rsc-callers");
    let symbols_root = analysis_fixture("tests-impact-symbol");
    let check_root = analysis_fixture("simple");

    let output = analyze_project_json_impl(
        json!({
            "root": queue_root,
            "reports": [
                { "type": "queues" },
                { "type": "playwrightCheck", "root": playwright_root },
                {
                    "type": "reactAnalyze",
                    "root": react_root,
                    "targets": ["app/components/Mixed.tsx"]
                },
                { "type": "serverRoutes", "root": server_root },
                { "type": "effects", "root": effects_root, "kind": "valkey", "entry": "app/server.ts" },
                { "type": "rscCallers", "root": rsc_root, "component": "app/ui/Button.tsx" },
                {
                    "type": "symbols",
                    "root": symbols_root,
                    "files": ["utils.mts"],
                    "mode": "signature-impact",
                    "symbol": "parseDate"
                },
                {
                    "type": "flow",
                    "root": symbols_root,
                    "target": "utils.mts",
                    "direction": "dependents",
                    "relationships": ["import"]
                },
                {
                    "type": "check",
                    "root": check_root
                }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let value = parse_json(output);

    assert_eq!(value["reports"].as_array().unwrap().len(), 9);
    assert_eq!(
        value["reports"]
            .as_array()
            .unwrap()
            .iter()
            .map(|report| report["type"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![
            "queues",
            "playwrightCheck",
            "reactAnalyze",
            "serverRoutes",
            "effects",
            "rscCallers",
            "symbols",
            "flow",
            "check",
        ]
    );
}
