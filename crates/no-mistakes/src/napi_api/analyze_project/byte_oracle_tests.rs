use super::*;
use std::cell::Cell;
thread_local! { static ORACLE_ENABLED: Cell<bool> = const { Cell::new(false) }; static OLD_TYPED_SERIALIZER: Cell<bool> = const { Cell::new(false) }; }

pub(super) fn json_raw_report<T: serde::Serialize + ?Sized>(value: &T) -> Box<RawValue> {
    if OLD_TYPED_SERIALIZER.get() {
        // Verbatim prior typed Value -> string -> RawValue serializer, including object order.
        let value =
            serde_json::to_value(value).expect("JSON serialization of Rust structs never fails");
        return RawValue::from_string(value.to_string())
            .expect("JSON Value re-serialize never fails");
    }
    serde_json::value::to_raw_value(value).expect("JSON serialization of Rust structs never fails")
}

pub(super) fn run_report(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
    context: &context::AnalyzeProjectContext,
) -> AnyhowResult<Box<RawValue>> {
    if !ORACLE_ENABLED.get() {
        return run_report_inner(request, options, context);
    }
    let new = run_report_inner(request, options, context)?;
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
    assert!(!old.get().is_empty());
    assert_eq!(
        old.get().as_bytes(),
        new.get().as_bytes(),
        "typed serializer raw bytes: {}",
        request.report_type
    );
    let make_outer = |result| AnalyzeProjectResult {
        reports: vec![AnalyzeReportResult {
            id: request.id.clone(),
            report_type: request.report_type.clone(),
            result,
        }],
    };
    assert_eq!(
        serde_json::to_vec(&make_outer(old)).unwrap(),
        serde_json::to_vec(&make_outer(new.clone())).unwrap(),
        "complete outer response bytes: {}",
        request.report_type
    );
    Ok(new)
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
                let result = run_report(request, &options, &context).unwrap();
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
