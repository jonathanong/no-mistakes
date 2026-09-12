use super::{AnalyzeProjectContext, AnalyzeProjectOptions, AnalyzeReportRequest};
use crate::codebase::dependencies::Direction;
use serde_json::{json, Value};

fn fixture_root(name: &str) -> String {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
    .display()
    .to_string()
}

fn options(root: &str, report: Value) -> (AnalyzeReportRequest, AnalyzeProjectOptions) {
    let options: AnalyzeProjectOptions = serde_json::from_value(json!({
        "root": root,
        "reports": [report]
    }))
    .unwrap();
    (options.reports[0].clone(), options)
}

#[test]
fn prepared_scope_report_helpers_surface_missing_inputs() {
    let root = fixture_root("simple");
    let prepared: AnalyzeProjectOptions = serde_json::from_value(json!({
        "root": root,
        "reports": [{ "type": "symbols", "files": ["a.mts"] }]
    }))
    .unwrap();
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();

    let (graph, graph_options) = options(
        &root,
        json!({ "type": "dependencies", "files": ["a.mts"], "filters": ["["] }),
    );
    let graph_error = context
        .graph_report(&graph, &graph_options, Direction::Deps)
        .unwrap_err();
    assert!(
        graph_error.to_string().contains("glob"),
        "{graph_error:#}"
    );

    let (imports, import_options) = options(
        &root,
        json!({ "type": "importUsages", "filters": ["a.mts"] }),
    );
    let import_error = context
        .import_usages_report(&imports, &import_options)
        .unwrap_err();
    assert!(
        import_error
            .to_string()
            .contains("prepared importUsages file universe is missing"),
        "{import_error:#}"
    );

    let (symbols, symbol_options) = options(
        &root,
        json!({ "type": "symbols", "files": ["a.mts"], "notAField": true }),
    );
    assert!(context.symbols_report(&symbols, &symbol_options).is_err());

    let (flow, flow_options) = options(&root, json!({ "type": "flow" }));
    let flow_error = context.flow_report(&flow, &flow_options).unwrap_err();
    assert!(
        flow_error
            .to_string()
            .contains("target is required for flow"),
        "{flow_error:#}"
    );

    let (flow_dir, flow_dir_options) = options(
        &root,
        json!({
            "type": "flow",
            "target": "a.mts",
            "direction": "sideways"
        }),
    );
    let flow_dir_error = context
        .flow_report(&flow_dir, &flow_dir_options)
        .unwrap_err();
    assert!(
        flow_dir_error
            .to_string()
            .contains("unknown flow direction"),
        "{flow_dir_error:#}"
    );

    let (effects, effects_options) = options(&root, json!({ "type": "effects" }));
    let effects_error = context
        .effects_report(&effects, &effects_options)
        .unwrap_err();
    assert!(
        effects_error
            .to_string()
            .contains("kind is required for effects"),
        "{effects_error:#}"
    );

    let (effects_entry, effects_entry_options) =
        options(&root, json!({ "type": "effects", "kind": "fetch" }));
    let entry_error = context
        .effects_report(&effects_entry, &effects_entry_options)
        .unwrap_err();
    assert!(
        entry_error
            .to_string()
            .contains("entry is required for effects"),
        "{entry_error:#}"
    );

    let (rsc, rsc_options) = options(&root, json!({ "type": "rscCallers" }));
    let rsc_error = context.rsc_callers_report(&rsc, &rsc_options).unwrap_err();
    assert!(
        rsc_error
            .to_string()
            .contains("component is required for rsc-callers"),
        "{rsc_error:#}"
    );
}

#[test]
fn prepared_flow_report_accepts_dependents_direction() {
    let root = fixture_root("tests-impact-symbol");
    let prepared: AnalyzeProjectOptions = serde_json::from_value(json!({
        "root": root,
        "reports": [{
            "type": "flow",
            "target": "utils.mts#parseDate",
            "direction": "dependents",
            "depth": 1,
            "relationships": ["import"]
        }]
    }))
    .unwrap();
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let report = context
        .flow_report(&prepared.reports[0], &prepared)
        .unwrap();
    assert_eq!(report["target"], "utils.mts#parseDate");
}
