use super::*;

#[test]
fn analyze_project_dispatches_all_domain_report_types() {
    for report_type in [
        "symbols",
        "flow",
        "effects",
        "rscCallers",
        "importUsages",
        "queueEdges",
        "queueRelated",
        "queueCheck",
        "serverRoutes",
        "serverRouteList",
        "serverRouteEdges",
        "serverRouteRelated",
        "serverContracts",
        "reactAnalyze",
        "reactCheck",
        "playwrightCheck",
        "playwrightEdges",
        "playwrightRelated",
        "playwrightTests",
        "check",
        "importers",
        "exportsOf",
        "deadExports",
        "callSites",
        "resolveCheck",
        "fetches",
        "dataPw",
        "registryExtension",
        "testsPlan",
        "testsImpact",
        "testsTargets",
        "testsWhy",
        "testsComment",
        "testsGraph",
        "testsGraphMermaid",
        "lockfileDiff",
        "ciImpact",
        "ciEnv",
        "ciTopology",
        "impactedChecks",
        "infraResourceRefs",
        "infraOutputs",
        "infraTestFor",
        "swiftImporters",
        "swiftTestTargets",
        "validateMermaidMarkdown",
    ] {
        let result = analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(json!({
                "root": fixture_root("simple"),
                "reports": [{
                    "type": report_type,
                    "id": report_type,
                    "files": ["a.mts"]
                }]
            })
            .to_string(),)
        );
        if let Err(error) = result {
            assert!(
                !error.reason.contains("unknown analyzeProject report type"),
                "{report_type} should be recognized"
            );
        }
    }
}
