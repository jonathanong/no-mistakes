#[test]
fn report_caches_call_each_analyzer_once_per_canonical_key() {
    let key = canonical_filter_key(&[
        "src/**".to_string(),
        "tests/**".to_string(),
        "src/**".to_string(),
    ])
    .unwrap();
    let equivalent_key =
        canonical_filter_key(&["tests/**".to_string(), "src/**".to_string()]).unwrap();

    for domain in ["queue", "server"] {
        let plain_calls = Cell::new(0);
        let indexed_calls = Cell::new(0);
        let plain = ReportCache::new(HashMap::new());
        let indexed = ReportCache::new(HashMap::new());

        for traversal in [false, false, true, true] {
            let report = cached_analysis(
                &plain,
                &indexed,
                if traversal { &equivalent_key } else { &key },
                traversal,
                &|| {
                    plain_calls.set(plain_calls.get() + 1);
                    Ok(format!("{domain}-plain"))
                },
                &|| {
                    indexed_calls.set(indexed_calls.get() + 1);
                    Ok(format!("{domain}-indexed"))
                },
            )
            .unwrap();
            match (traversal, report) {
                (false, CachedAnalysis::Plain(report)) => {
                    assert_eq!(report, format!("{domain}-plain"));
                }
                (true, CachedAnalysis::Indexed(report)) => {
                    assert_eq!(report, format!("{domain}-indexed"));
                }
                _ => panic!("{domain} selected the wrong analyzer"),
            }
        }

        assert_eq!(plain_calls.get(), 1, "{domain} plain analyzer");
        assert_eq!(indexed_calls.get(), 1, "{domain} indexed analyzer");
    }
}

#[test]
fn omitted_automatic_and_explicit_tsconfig_use_separate_scopes() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/forbidden-dependencies-passes/fixture"),
    );
    let options: super::AnalyzeProjectOptions = serde_json::from_value(serde_json::json!({
        "root": root,
        "reports": [
            { "type": "check" },
            {
                "type": "check",
                "config": ".no-mistakes.yml",
                "tsconfig": "tsconfig.json"
            }
        ]
    }))
    .unwrap();

    let context = super::AnalyzeProjectContext::prepare(&options).unwrap();
    assert_eq!(context.scopes.len(), 2);
    assert_eq!(context.scope_aliases.len(), 2);
    let mut automatic_modes = context
        .scopes
        .keys()
        .map(|scope| scope.automatic_tsconfig)
        .collect::<Vec<_>>();
    automatic_modes.sort();
    assert_eq!(automatic_modes, vec![false, true]);
}

include!("../report_error_tests.rs");
include!("../report_success_tests.rs");

#[test]
fn check_report_returns_a_rule_option_error() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/check-rule-error/fixture"),
    );
    let options: super::super::types::AnalyzeProjectOptions =
        serde_json::from_value(serde_json::json!({
            "root": root,
            "config": "broken.yml",
            "reports": [{"type": "check"}]
        }))
        .unwrap();
    let context = super::AnalyzeProjectContext::prepare(&options).unwrap();
    let error = context
        .project_report(&options.reports[0], &options)
        .expect_err("missing schema catalog");
    assert!(
        error.to_string().contains("schemaCatalogPath: required"),
        "{error:#}"
    );
}

#[test]
fn command_report_rejects_unknown_type() {
    let request = super::super::types::AnalyzeReportRequest {
        id: None,
        report_type: "missing".to_string(),
        options: serde_json::Map::new(),
    };
    let options = super::super::types::AnalyzeProjectOptions {
        root: None,
        tsconfig: None,
        config: None,
        filters: Vec::new(),
        reports: Vec::new(),
    };
    let error = super::run_command_report(&request, &options).unwrap_err();
    assert!(error
        .to_string()
        .contains("unknown analyzeProject report type: missing"));
}
