use super::super::analyze_project_json_impl;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn constructor_fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/unconstructed-error-class/construction-targets/fixture"),
    )
}

fn constructor_check(root: &Path, config: &str) -> Value {
    serde_json::from_str(
        &crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root":root,"config":root.join(config),"includeSuppressed":true}).to_string(),
        ))
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn constructor_policy_matches_aggregate_api_and_preserves_ordinary_queries() {
    let root = constructor_fixture();
    for config in [".no-mistakes.yml", "report.yml", "advise.yml", "ignore.yml"] {
        let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({
                "root": root, "config": root.join(config),
                "reports": [{"type":"check", "includeSuppressed":true}],
            })
            .to_string(),
        ))
        .unwrap();
        let project: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            project["reports"][0]["result"],
            constructor_check(&root, config)
        );
    }
    let analyze = |check| {
        let mut reports = vec![
            serde_json::json!({"type":"importers", "file":"src/plain.ts"}),
            serde_json::json!({"type":"callSites", "file":"src/errors.ts", "exportName":"UsedError"}),
        ];
        if check {
            reports.push(serde_json::json!({"type":"check"}));
        }
        let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            serde_json::json!({"root":root,"reports":reports}).to_string(),
        ))
        .unwrap();
        serde_json::from_str::<serde_json::Value>(&output).unwrap()
    };
    let ordinary = analyze(false);
    let combined = analyze(true);
    assert!(ordinary.to_string().contains("main.ts"));
    for index in 0..2 {
        assert_eq!(ordinary["reports"][index], combined["reports"][index]);
    }
}
