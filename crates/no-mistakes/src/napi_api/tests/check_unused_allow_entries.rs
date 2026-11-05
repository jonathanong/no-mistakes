use super::*;
use serde_json::{json, Value};

#[test]
fn stale_allow_entries_are_advisory_and_match_batched_check() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/unused-allow-entries");
    let report: Value = serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root}).to_string(),
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(report["rules"], json!([]), "{report}");
    let advice = report["advisories"].as_array().unwrap();
    assert_eq!(advice.len(), 1, "{report}");
    assert_eq!(advice[0]["rule"], "unused-allow-entry");
    assert_eq!(advice[0]["file"], ".no-mistakes.yml");
    assert_eq!(advice[0]["line"], 13);
    assert_eq!(advice[0]["import"], "query-reached-per-item");
    assert_eq!(advice[0]["target"], "src/entry.mts#retired");
    let batch: Value = serde_json::from_str(
        &crate::napi_api::analyze_project::analyze_project_json_impl(
            crate::napi_api::options::test_json_arg(
                json!({"root": root, "reports": [{"type": "check"}]}).to_string(),
            ),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(batch["reports"][0]["result"], report);
}
