use super::*;

#[test]
fn repeated_check_only_reports_share_the_prepared_canonical_graph() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("test-no-unmocked-dynamic-imports"),
            "config": ".no-mistakes-combined.yml",
            "reports": [{"type": "check"}, {"type": "check"}]
        })
        .to_string(),
    )
    .unwrap();
    let context = context::AnalyzeProjectContext::prepare(&options).unwrap();
    let first = report_value(&run_report(&options.reports[0], &options, &context).unwrap());
    let second = report_value(&run_report(&options.reports[1], &options, &context).unwrap());
    assert_eq!(first, second);
    assert_eq!(context.graph_build_count(), 1);
}

#[test]
fn repeated_partial_checks_share_the_complete_import_projection() {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": fixture_root("foreign-receiver-work-budget"),
            "config": "import-mixed-config.yml",
            "reports": [{"type": "check"}, {"type": "check"}]
        })
        .to_string(),
    )
    .unwrap();
    let context = context::AnalyzeProjectContext::prepare(&options).unwrap();
    let first = report_value(&run_report(&options.reports[0], &options, &context).unwrap());
    let second = report_value(&run_report(&options.reports[1], &options, &context).unwrap());
    assert_eq!(first, second);
    assert!(!first["incomplete"].as_array().unwrap().is_empty());
    assert!(!first["rules"].as_array().unwrap().is_empty());
    assert_eq!(context.graph_build_count(), 1);
}
