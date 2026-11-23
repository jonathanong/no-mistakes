use serde_json::{json, Value};
use std::path::PathBuf;

#[test]
fn suppression_index_keeps_check_and_analyze_project_reports_identical() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/forbidden-calls/suppression-index/fixture");
    let standalone: Value = serde_json::from_str(
        &super::check_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root, "includeSuppressed": true}).to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    let aggregate: Value = serde_json::from_str(
        &super::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(
                json!({"root": root, "reports": [{"type": "check", "includeSuppressed": true}]})
                    .to_string(),
            ),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(standalone["rules"].as_array().unwrap().len(), 2);
    assert_eq!(standalone["suppressed"].as_array().unwrap().len(), 3);
    assert_eq!(aggregate["reports"][0]["result"], standalone);
}
