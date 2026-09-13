#[test]
fn prepared_scope_report_helpers_cover_success_and_duplicate_import_keys() {
    let root = fixture_root("simple");
    let abs = PathBuf::from(&root).join("a.mts");
    let prepared: AnalyzeProjectOptions = serde_json::from_value(json!({
        "root": root,
        "reports": [
            { "type": "symbols", "files": [abs] },
            { "type": "dependencies", "files": ["a.mts"], "relationships": ["import"] },
            { "type": "importUsages", "id": "first" },
            { "type": "importUsages", "id": "second" },
            { "type": "rscCallers" }
        ]
    }))
    .unwrap();
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();

    let symbols = context
        .symbols_report(&prepared.reports[0], &prepared)
        .unwrap();
    assert!(symbols.is_object());

    let graph = context
        .graph_report(&prepared.reports[1], &prepared, Direction::Deps)
        .unwrap();
    assert!(!graph.get().is_empty());

    let first = context
        .import_usages_report(&prepared.reports[2], &prepared)
        .unwrap();
    let second = context
        .import_usages_report(&prepared.reports[3], &prepared)
        .unwrap();
    assert_eq!(first, second);
}

#[test]
fn prepared_symbols_signature_impact_uses_the_traversal_helper() {
    let root = fixture_root("tests-impact-symbol");
    let prepared: AnalyzeProjectOptions = serde_json::from_value(json!({
        "root": root,
        "reports": [{
            "type": "symbols",
            "mode": "signature-impact",
            "files": ["utils.mts"],
            "symbol": "parseDate"
        }]
    }))
    .unwrap();
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let report = context
        .symbols_report(&prepared.reports[0], &prepared)
        .unwrap();
    assert!(report.is_object());
}
