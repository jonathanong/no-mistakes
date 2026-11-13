use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Output {
    reports: Vec<Report>,
}

#[derive(Deserialize)]
struct Report {
    id: Option<String>,
    #[serde(rename = "type")]
    report_type: String,
    result: Box<RawValue>,
}

#[test]
fn saved_typed_option_requests_preserve_standalone_report_bytes() {
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let matrix: Vec<Value> = serde_json::from_str(include_str!(
        "../../../../../test-cases/codebase-analysis/analyze-typed-options/requests.json"
    ))
    .unwrap();
    let mut covered = std::collections::BTreeSet::new();
    for case in matrix {
        let root = crate::codebase::ts_resolver::normalize_path(
            &repo.join(case["root"].as_str().unwrap()),
        );
        let mut batch = case["batch"].clone();
        batch["root"] = Value::String(root.display().to_string());
        let mut standalone = case["standalone"].clone();
        standalone["root"] = batch["root"].clone();
        let report_type = batch["reports"][0]["type"].as_str().unwrap();
        let expected = match report_type {
            "symbols" => crate::napi_api::symbols_json_impl(standalone),
            "flow" => crate::napi_api::flow_json_impl(standalone),
            "queues" => crate::napi_api::queues_json_impl(standalone),
            "playwrightCheck" => crate::napi_api::playwright_check_json_impl(standalone),
            other => panic!("unmapped saved report type {other}"),
        }
        .unwrap();
        let output = analyze_project_value_impl(batch.clone()).unwrap();
        let output: Output = serde_json::from_str(&output).unwrap();
        assert_eq!(output.reports.len(), 1);
        let report = &output.reports[0];
        assert_eq!(report.report_type, report_type);
        assert_eq!(report.id.as_deref(), batch["reports"][0]["id"].as_str());
        assert!(
            report
                .result
                .get()
                .contains(case["evidence"].as_str().unwrap()),
            "{report_type}: {}",
            report.result.get()
        );
        assert_eq!(
            report.result.get().as_bytes(),
            expected.trim().as_bytes(),
            "{report_type} public report bytes"
        );
        covered.insert(report_type.to_owned());
    }
    assert_eq!(
        covered,
        ["flow", "playwrightCheck", "queues", "symbols"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    );
}

#[test]
fn import_usage_cache_keys_keep_original_merged_json_and_filter_presence() {
    let mut fixture: Value = serde_json::from_str(include_str!(
        "../../../../../test-cases/codebase-analysis/analyze-typed-options/import-usage-keys.json"
    ))
    .unwrap();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/simple/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root)
        .display()
        .to_string();
    fixture["options"]["root"] = Value::String(root.clone());
    let options: AnalyzeProjectOptions =
        serde_json::from_value(fixture["options"].clone()).unwrap();
    let mut keys = Vec::new();
    for (request, expected) in options
        .reports
        .iter()
        .zip(fixture["expected"].as_array().unwrap())
    {
        let mut expected = expected.clone();
        expected["root"] = Value::String(root.clone());
        let key = options::import_usages_options(request, &options).unwrap();
        assert_eq!(key, crate::cli::json_string(&expected));
        keys.push(key);
    }
    assert_eq!(keys.len(), 2);
    assert_ne!(keys[0], keys[1]);
}
