use super::super::{context, AnalyzeProjectOptions, Value};
use super::{literal_report_bytes, run_report, symbols_value_serializer_runs, ORACLE_ENABLED};
use serde_json::value::RawValue;

struct ClearOracle;
impl Drop for ClearOracle {
    fn drop(&mut self) {
        ORACLE_ENABLED.set(false);
        super::OLD_TYPED_SERIALIZER.set(false);
    }
}

fn repo_fixture(relative: &str) -> String {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
        .to_string_lossy()
        .into_owned()
}

/// One prepared context. The original serializer only reparses that report's writer JSON.
fn assert_symbols_branch(root: &str, report: Value) -> Value {
    let _guard = ClearOracle;
    let options: AnalyzeProjectOptions = serde_json::from_value(serde_json::json!({
        "root": repo_fixture(root),
        "reports": [report]
    }))
    .unwrap();
    let context = context::AnalyzeProjectContext::prepare(&options).unwrap();
    let runs_before = symbols_value_serializer_runs();
    ORACLE_ENABLED.set(true);
    let direct = run_report(&options.reports[0], &options, &context).unwrap();
    ORACLE_ENABLED.set(false);
    assert_eq!(
        symbols_value_serializer_runs(),
        runs_before + 1,
        "the raw symbols branch must execute the original Value serializer"
    );
    // CI runs these tests with `--all-features`. That set includes
    // mermaid-validation, which enables serde_json/preserve_order, so writer
    // order starts at roots and these object keys are not sorted.
    assert!(
        direct.get().starts_with("{\"roots\":"),
        "CI configuration is --all-features, which enables serde_json/preserve_order; symbols keys are not sorted: {}",
        direct.get()
    );
    serde_json::from_str(direct.get()).unwrap()
}

#[test]
fn all_features_include_serde_json_preserve_order() {
    // CI runs these tests with `--all-features`, including mermaid-validation,
    // which enables serde_json/preserve_order.
    let raw = r#"{"z":1,"a":2}"#;
    let value: Value = serde_json::from_str(raw).unwrap();
    assert_eq!(
        value.to_string(),
        raw,
        "CI configuration is --all-features, which enables serde_json/preserve_order; object keys are not sorted"
    );
}

#[test]
fn ordinary_symbols_match_original_value_serializer() {
    let parsed = assert_symbols_branch(
        "test-cases/codebase-analysis/simple/fixture",
        serde_json::json!({
            "type": "symbols",
            "id": "symbols",
            "files": ["a.mts"]
        }),
    );
    assert!(parsed["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| !file["exports"].as_array().unwrap().is_empty()));
}

#[test]
fn signature_impact_symbols_match_original_value_serializer() {
    let parsed = assert_symbols_branch(
        "test-cases/codebase-analysis/tests-impact-symbol/fixture",
        serde_json::json!({
            "type": "symbols",
            "id": "signature-impact",
            "mode": "signature-impact",
            "files": ["utils.mts"],
            "symbol": "parseDate"
        }),
    );
    assert_eq!(parsed["symbol"], "parseDate");
    assert!(!parsed["testCallers"].as_array().unwrap().is_empty());
}

#[test]
fn symbols_byte_oracle_rejects_value_equal_slash_escape() {
    // Saved negative control. Equal parsed values and equal compact JSON still
    // differ in literal bytes; canonicalizing before the compare would hide that.
    let control: Value = serde_json::from_str(include_str!(
        "../../../../../test-cases/codebase-analysis/analyze-serialize-once/symbols-byte-oracle-negative-control.json"
    ))
    .unwrap();
    let left = control["left"].as_str().unwrap();
    let right = control["right"].as_str().unwrap();
    let left_value: Value = serde_json::from_str(left).unwrap();
    let right_value: Value = serde_json::from_str(right).unwrap();
    assert_eq!(left_value, right_value);
    assert_eq!(left_value.to_string(), right_value.to_string());
    assert_ne!(left.as_bytes(), right.as_bytes());
    let left_raw = RawValue::from_string(left.to_string()).unwrap();
    let right_raw = RawValue::from_string(right.to_string()).unwrap();
    let bytes = literal_report_bytes(
        "symbols",
        &Some("negative-control".to_string()),
        &left_raw,
        &right_raw,
    );
    assert_ne!(bytes.raw_old, bytes.raw_new);
    assert_ne!(bytes.outer_old, bytes.outer_new);
    // Different bytes are not enough: weakening the assert to a parsed-value
    // comparison would still see equal values and would not panic.
    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        super::assert_literal_report_bytes(
            "symbols",
            &Some("negative-control".to_string()),
            &left_raw,
            &right_raw,
        );
    }));
    let message = panic_message(panicked.expect_err("byte oracle must reject the slash escape"));
    assert!(
        message.contains("typed serializer raw bytes")
            || message.contains("complete outer response bytes"),
        "byte assert must reject the slash escape, got {message}"
    );
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|message| (*message).to_string())
        })
        .unwrap_or_else(|| "<non-string panic>".to_string())
}
