use super::*;
use serde_json::{json, Value};
use std::path::PathBuf;

fn playwright_prewarm_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/parser-count/playwright"),
    )
}

#[test]
fn analyze_project_prewarms_shared_playwright_analysis_before_parallel_reports() {
    let root = playwright_prewarm_fixture();
    let root_json = root.display().to_string();
    let request = json!({
        "root": root_json,
        "reports": [
            { "type": "playwrightCheck" },
            { "type": "playwrightEdges" },
            { "type": "playwrightRelated", "files": ["app/page.tsx"] },
            { "type": "playwrightTests", "files": ["app/page.tsx"] }
        ]
    });
    let standalone = [
        crate::napi_api::playwright_check_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root_json }).to_string(),
        ))
        .unwrap(),
        crate::napi_api::playwright_edges_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root_json }).to_string(),
        ))
        .unwrap(),
        crate::napi_api::playwright_related_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root_json, "files": ["app/page.tsx"] }).to_string(),
        ))
        .unwrap(),
        crate::napi_api::playwright_tests_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root_json, "files": ["app/page.tsx"] }).to_string(),
        ))
        .unwrap(),
    ]
    .map(|value| serde_json::from_str::<Value>(&value).unwrap());
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();
    let options = parse_options_value::<AnalyzeProjectOptions>(request.clone()).unwrap();
    let context = pool
        .install(|| context::AnalyzeProjectContext::prepare(&options))
        .unwrap();
    assert_eq!(context.initialized_playwright_analysis_count(), 1);
    let output = pool
        .install(|| {
            analyze_project_json_impl(crate::napi_api::options::test_json_arg(request.to_string()))
        })
        .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();

    for (index, expected) in standalone.iter().enumerate() {
        assert_eq!(&value["reports"][index]["result"], expected);
    }
}
