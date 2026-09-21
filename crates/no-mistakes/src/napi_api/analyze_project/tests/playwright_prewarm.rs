use super::*;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use wait_timeout::ChildExt;

fn playwright_prewarm_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/parser-count/playwright"),
    )
}

fn playwright_prewarm_request() -> Value {
    let root = playwright_prewarm_fixture();
    json!({
        "root": root.display().to_string(),
        "reports": [
            { "type": "playwrightCheck" },
            { "type": "playwrightEdges" },
            { "type": "playwrightRelated", "files": ["app/page.tsx"] },
            { "type": "playwrightTests", "files": ["app/page.tsx"] }
        ]
    })
}

fn assert_playwright_prewarm_dispatch() {
    let request = playwright_prewarm_request();
    let root_json = request["root"].as_str().unwrap();
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
    let options = parse_options_value::<AnalyzeProjectOptions>(request).unwrap();
    let context = pool
        .install(|| context::AnalyzeProjectContext::prepare(&options))
        .unwrap();
    assert_eq!(context.initialized_playwright_analysis_count(), 1);
    let output = pool
        .install(|| analyze_project_with_context(&options, &context))
        .unwrap();
    let value = serde_json::to_value(output).unwrap();

    for (index, expected) in standalone.iter().enumerate() {
        assert_eq!(&value["reports"][index]["result"], expected);
    }
}

#[test]
fn analyze_project_prewarms_shared_playwright_analysis_before_parallel_reports() {
    if std::env::var_os("NO_MISTAKES_PLAYWRIGHT_PREWARM_SUBPROCESS").is_some() {
        assert_playwright_prewarm_dispatch();
        return;
    }

    let mut child = Command::new(std::env::current_exe().unwrap());
    child
        .args([
            "--exact",
            "napi_api::analyze_project::playwright_prewarm_tests::analyze_project_prewarms_shared_playwright_analysis_before_parallel_reports",
        ])
        .env("NO_MISTAKES_PLAYWRIGHT_PREWARM_SUBPROCESS", "1");
    let mut child = child.spawn().unwrap();
    let Some(status) = child.wait_timeout(Duration::from_secs(60)).unwrap() else {
        let _ = child.kill();
        let _ = child.wait();
        panic!("Playwright prewarm dispatch subprocess did not complete within 60 seconds");
    };
    assert!(
        status.success(),
        "Playwright prewarm subprocess failed: {status}"
    );
}
