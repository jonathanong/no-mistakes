use super::*;
use serde_json::{json, Value};

#[test]
fn check_timing_report_is_opt_in_and_keeps_findings() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/forbidden-dependencies-basic/fixture");
    let run = |timings| -> Value {
        serde_json::from_str(&check_json_impl(json!({"root": root, "timings": timings})).unwrap())
            .unwrap()
    };
    let plain = run(false);
    let mut timed = run(true);
    assert!(plain.get("timings").is_none());
    let timings = timed.as_object_mut().unwrap().remove("timings").unwrap();
    let entries = timings.as_array().unwrap();
    assert!(entries
        .iter()
        .any(|entry| entry["label"] == "analysis.rules.forbidden-dependencies"));
    assert!(entries
        .iter()
        .all(|entry| entry["durationMs"].is_number() && entry["parallel"].is_boolean()));
    assert_eq!(plain, timed);
}
