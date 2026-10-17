use super::*;

pub(super) fn run_report(
    request: &AnalyzeReportRequest,
    options: &AnalyzeProjectOptions,
    context: &context::AnalyzeProjectContext,
) -> AnyhowResult<Box<RawValue>> {
    run_report_inner(request, options, context)
}

/// One compact serde pass. `RawValue` embeds these bytes in the outer result.
pub(super) fn json_raw_report<T: serde::Serialize + ?Sized>(value: &T) -> Box<RawValue> {
    serde_json::value::to_raw_value(value).expect("JSON serialization of Rust structs never fails")
}

/// Writer JSON is already compact, plus a trailing newline that `RawValue` drops.
pub(super) fn json_raw_text(output: String) -> AnyhowResult<Box<RawValue>> {
    Ok(RawValue::from_string(output)?)
}
