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

#[test]
fn caller_owned_check_and_ordinary_projection_share_one_graph_and_parse() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/test-no-live-repository-scan/caller-roots/imported-config-unstub"),
    );
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({
            "root": root,
            "config": ".no-mistakes.yml",
            "reports": [
                {"type": "dependents", "files": ["settings.ts"], "relationships": ["import"], "depth": 3},
                {"type": "check"},
                {"type": "check"}
            ]
        }).to_string(),
    ).unwrap();
    crate::ast::begin_parse_count(&root);
    let context = context::AnalyzeProjectContext::prepare(&options).unwrap();
    let results: Vec<Value> = options
        .reports
        .iter()
        .map(|request| report_value(&run_report(request, &options, &context).unwrap()))
        .collect();
    let counts = crate::ast::finish_parse_count(&root);
    assert!(results[0]["files"].as_array().unwrap().is_empty());
    assert_eq!(results[1], results[2]);
    assert_eq!(results[1]["advisories"][0]["file"], "src/list.ts");
    assert_eq!(results[1]["advisories"][0]["line"], 4);
    assert_eq!(context.graph_build_count(), 1);
    for file in [
        "qa.runner.ts",
        "settings.ts",
        "setup.ts",
        "src/config-client.ts",
        "src/list.ts",
        "tests/t.test.ts",
    ] {
        assert_eq!(
            counts.get(&root.join(file)),
            Some(&1),
            "{file}: {counts:#?}"
        );
    }
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
}

#[test]
fn caller_owned_check_preserves_ordinary_config_diagnostics() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/test-no-live-repository-scan/caller-roots/projection-config-diagnostics"),
    );
    for (kind, file, explicit) in [
        ("dependents", "settings.ts", false),
        ("dependencies", "src/list.ts", false),
        ("dependencies", "src/config-client.ts", false),
        ("dependents", "qa.runner.ts", true),
    ] {
        let report =
            json!({"type": kind, "files": [file], "relationships": ["import"], "depth": 3});
        let before = prepared_projection_results(&root, vec![report.clone()]);
        let after = prepared_projection_results(&root, vec![report, json!({"type": "check"})]);
        assert_eq!(before[0], after[0], "{kind}: {file}");
        let expected = if explicit {
            vec![json!("qa.runner.ts")]
        } else {
            vec![]
        };
        assert_eq!(
            before[0]["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row["file"].clone())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(after[1]["advisories"][0]["file"], "src/list.ts");
    }
    for report in [
        json!({"type": "dependents", "files": [{"file": "settings.ts", "symbol": "resetPolicy"}], "relationships": ["import"], "depth": 3}),
        json!({"type": "dependents", "files": ["settings.ts"], "includeSymbols": true, "relationships": ["import"], "depth": 3}),
    ] {
        let before = prepared_projection_results(&root, vec![report.clone()]);
        let after = prepared_projection_results(&root, vec![report, json!({"type": "check"})]);
        assert_eq!(before[0], after[0]);
        assert!(before[0]["files"].as_array().unwrap().is_empty());
    }
}

fn prepared_projection_results(root: &std::path::Path, reports: Vec<Value>) -> Vec<Value> {
    let options = parse_options::<AnalyzeProjectOptions>(
        &json!({"root": root, "config": ".no-mistakes.yml", "reports": reports}).to_string(),
    )
    .unwrap();
    let context = context::AnalyzeProjectContext::prepare(&options).unwrap();
    options
        .reports
        .iter()
        .map(|request| report_value(&run_report(request, &options, &context).unwrap()))
        .collect()
}
