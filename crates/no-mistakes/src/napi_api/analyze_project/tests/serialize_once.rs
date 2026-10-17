use crate::codebase::import_usages::ImportUsagesArgs;
use serde::Deserialize;

#[derive(Deserialize)]
struct AnalyzeOutput {
    reports: Vec<AnalyzeReport>,
}

#[derive(Deserialize)]
struct AnalyzeReport {
    id: Option<String>,
    #[serde(rename = "type")]
    report_type: String,
    result: Box<serde_json::value::RawValue>,
}

#[test]
fn import_usages_json_matches_the_value_round_trip() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/import-usages/fixture"),
    );
    let report = crate::codebase::import_usages::collect(&ImportUsagesArgs {
        files: Vec::new(),
        root: Some(root.clone()),
        scan_roots: Vec::new(),
        filters: vec!["src/main.mts".to_string()],
        format: None,
        json: true,
        timings: false,
    })
    .unwrap();
    let previous = serde_json::to_string(&serde_json::to_value(&report).unwrap()).unwrap();
    assert!(
        previous.contains("require-resolve"),
        "fixture must exercise a real import usage report"
    );

    let output = super::analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [{
                "type": "importUsages",
                "id": "imports",
                "filters": ["src/main.mts"]
            }]
        })
        .to_string(),
    ))
    .unwrap();
    let parsed: AnalyzeOutput = serde_json::from_str(&output).unwrap();
    let report = &parsed.reports[0];
    assert_eq!(report.id.as_deref(), Some("imports"));
    assert_eq!(report.report_type, "importUsages");
    assert_eq!(report.result.get(), previous);
}
