use super::*;
use std::cell::Cell;
thread_local! { static ORACLE_ENABLED: Cell<bool> = const { Cell::new(false) }; static OLD_TYPED_SERIALIZER: Cell<bool> = const { Cell::new(false) }; static SYMBOLS_VALUE_SERIALIZER_RUNS: Cell<u32> = const { Cell::new(0) }; }

pub(super) fn json_raw_report<T: serde::Serialize + ?Sized>(value: &T) -> Box<RawValue> {
    if OLD_TYPED_SERIALIZER.get() {
        // Verbatim prior typed Value -> string -> RawValue serializer, including object order.
        let value =
            serde_json::to_value(value).expect("JSON serialization of Rust structs never fails");
        return RawValue::from_string(value.to_string())
            .expect("JSON Value re-serialize never fails");
    }
    super::report_dispatch::json_raw_report(value)
}

pub(super) fn json_raw_text(output: String) -> AnyhowResult<Box<RawValue>> {
    if OLD_TYPED_SERIALIZER.get() {
        // Prior symbols path: parse writer JSON to Value, compact stringify, wrap as RawValue.
        // `--all-features` includes mermaid-validation, which enables
        // serde_json/preserve_order, so this does not sort object keys.
        SYMBOLS_VALUE_SERIALIZER_RUNS.set(SYMBOLS_VALUE_SERIALIZER_RUNS.get() + 1);
        let value = serde_json::from_str::<Value>(&output)
            .expect("symbols raw JSON parses for the original serializer");
        return Ok(
            RawValue::from_string(value.to_string()).expect("JSON Value re-serialize never fails")
        );
    }
    super::report_dispatch::json_raw_text(output)
}

pub(super) fn run_report(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
    context: &context::AnalyzeProjectContext,
) -> AnyhowResult<Box<RawValue>> {
    if !ORACLE_ENABLED.get() {
        return super::report_dispatch::run_report(request, options, context);
    }
    let new = super::report_dispatch::run_report(request, options, context)?;
    OLD_TYPED_SERIALIZER.set(true);
    let old_result = run_report_inner(request, options, context);
    OLD_TYPED_SERIALIZER.set(false);
    let old = old_result?;
    eprintln!(
        "BYTE_ORACLE_REPORT type={} old_bytes={} new_bytes={}",
        request.report_type,
        old.get().len(),
        new.get().len()
    );
    assert_literal_report_bytes(&request.report_type, &request.id, &old, &new);
    Ok(new)
}

pub(super) struct LiteralReportBytes {
    pub(super) raw_old: Vec<u8>,
    pub(super) raw_new: Vec<u8>,
    pub(super) outer_old: Vec<u8>,
    pub(super) outer_new: Vec<u8>,
}

pub(super) fn literal_report_bytes(
    report_type: &str,
    id: &Option<String>,
    old: &RawValue,
    new: &RawValue,
) -> LiteralReportBytes {
    LiteralReportBytes {
        raw_old: old.get().as_bytes().to_vec(),
        raw_new: new.get().as_bytes().to_vec(),
        outer_old: outer_response_bytes(report_type, id, old),
        outer_new: outer_response_bytes(report_type, id, new),
    }
}

fn assert_literal_report_bytes(
    report_type: &str,
    id: &Option<String>,
    old: &RawValue,
    new: &RawValue,
) {
    let bytes = literal_report_bytes(report_type, id, old, new);
    assert!(!bytes.raw_old.is_empty());
    assert_eq!(
        bytes.raw_old, bytes.raw_new,
        "typed serializer raw bytes: {report_type}"
    );
    assert_eq!(
        bytes.outer_old, bytes.outer_new,
        "complete outer response bytes: {report_type}"
    );
}

fn outer_response_bytes(report_type: &str, id: &Option<String>, result: &RawValue) -> Vec<u8> {
    serde_json::to_vec(&AnalyzeProjectResult {
        reports: vec![AnalyzeReportResult {
            id: id.clone(),
            report_type: report_type.to_string(),
            result: result.to_owned(),
        }],
    })
    .expect("outer analyzeProject response serializes")
}

fn symbols_value_serializer_runs() -> u32 {
    SYMBOLS_VALUE_SERIALIZER_RUNS.get()
}

#[test]
fn all_23_changed_report_families_preserve_literal_bytes() {
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let matrix: Vec<Value> = serde_json::from_str(include_str!(
        "../../../../../test-cases/codebase-analysis/analyze-serialize-once/requests.json"
    ))
    .unwrap();
    let mut covered = std::collections::BTreeSet::new();
    for mut group in matrix {
        group["root"] = Value::String(
            repo.join(group["root"].as_str().unwrap())
                .to_string_lossy()
                .into_owned(),
        );
        let options: AnalyzeProjectOptions = serde_json::from_value(group).unwrap();
        let context = context::AnalyzeProjectContext::prepare(&options).unwrap();
        ORACLE_ENABLED.set(true);
        let new_reports = options
            .reports
            .iter()
            .map(|request| {
                let runs_before = symbols_value_serializer_runs();
                let result = run_report(request, &options, &context).unwrap();
                if request.report_type == "symbols" {
                    assert!(
                        symbols_value_serializer_runs() > runs_before,
                        "OLD_TYPED_SERIALIZER must reach the raw symbols Value serializer"
                    );
                }
                covered.insert(request.report_type.clone());
                AnalyzeReportResult {
                    id: request.id.clone(),
                    report_type: request.report_type.clone(),
                    result,
                }
            })
            .collect();
        ORACLE_ENABLED.set(false);
        OLD_TYPED_SERIALIZER.set(true);
        let old_reports = options
            .reports
            .iter()
            .map(|request| AnalyzeReportResult {
                id: request.id.clone(),
                report_type: request.report_type.clone(),
                result: run_report_inner(request, &options, &context).unwrap(),
            })
            .collect();
        OLD_TYPED_SERIALIZER.set(false);
        assert_eq!(
            serde_json::to_vec(&AnalyzeProjectResult {
                reports: old_reports
            })
            .unwrap(),
            serde_json::to_vec(&AnalyzeProjectResult {
                reports: new_reports
            })
            .unwrap()
        );
    }
    assert_eq!(covered.len(), 24); // 23 changed families plus dependency input for derived resolve.
    assert!(covered.contains("playwrightRelated"));
}

#[path = "byte_oracle_symbols_tests.rs"]
mod symbols_byte_oracle;
