use super::*;

#[test]
fn graph_reports_honor_per_report_scope_overrides() {
    let output = analyze_project_json_impl(
        json!({
            "root": fixture_root("exports"),
            "reports": [{
                "type": "dependencies",
                "root": fixture_root("simple"),
                "files": ["a.mts"]
            }]
        })
        .to_string(),
    )
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    assert!(value["reports"][0]["result"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| file["path"] == "b.mts"));
}

#[test]
fn graph_reports_surface_traversal_errors() {
    let error = analyze_project_json_impl(
        json!({
            "root": fixture_root("simple"),
            "filters": ["["],
            "reports": [{ "type": "dependencies", "files": ["a.mts"] }]
        })
        .to_string(),
    )
    .unwrap_err();
    assert!(error.reason.contains("glob"));
}

#[test]
fn shared_graph_context_builds_once_for_multiple_graph_reports() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("simple"),
            "reports": [
                { "type": "dependencies", "files": ["a.mts"], "relationships": ["import"] },
                { "type": "dependents", "files": ["b.mts"], "relationships": ["import"] }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let mut shared = prepare_shared_traversal(&options).unwrap().unwrap();
    for request in &options.reports {
        let direction = if request.report_type == "dependencies" {
            Direction::Deps
        } else {
            Direction::Dependents
        };
        let _ = graph_report(request, &options, direction, Some(&mut shared)).unwrap();
    }
    assert_eq!(shared.graph_builds, 1);
}

#[test]
fn shared_graph_context_keeps_import_only_dependencies_lazy() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("simple"),
            "reports": [
                { "type": "dependencies", "files": ["a.mts"], "relationships": ["import"] }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let mut shared = prepare_shared_traversal(&options).unwrap().unwrap();
    let result = graph_report(
        &options.reports[0],
        &options,
        Direction::Deps,
        Some(&mut shared),
    )
    .unwrap();
    assert_eq!(shared.graph_builds, 0);
    assert!(result["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| { file["path"] == "b.mts" }));
}

#[test]
fn shared_import_usages_context_reuses_collected_facts() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("import-usages"),
            "reports": [
                { "type": "importUsages", "filters": ["src/main.mts"] }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let mut shared = prepare_shared_traversal(&options).unwrap().unwrap();
    assert!(!shared.facts().is_empty());
    assert!(!shared.facts().is_empty());
}

#[test]
fn shared_graph_context_supports_symbol_dependents() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("simple"),
            "reports": [
                { "type": "dependents", "files": ["b.mts#b"], "relationships": ["import"] }
            ]
        })
        .to_string(),
    )
    .unwrap();
    let mut shared = prepare_shared_traversal(&options).unwrap().unwrap();
    let result = graph_report(
        &options.reports[0],
        &options,
        Direction::Dependents,
        Some(&mut shared),
    )
    .unwrap();
    assert_eq!(shared.graph_builds, 1);
    assert!(result["files"].is_array());
}

include!("../tests/symbol_reports.rs");
