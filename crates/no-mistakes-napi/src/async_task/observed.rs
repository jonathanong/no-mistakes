use super::{to_invocation_napi_error, to_napi_error};
use napi::bindgen_prelude::Buffer;
use no_mistakes::diagnostics::{DiagnosticRequestGuard, TimingKind};
use serde_json::Value;

pub(super) const DIAGNOSTICS_JSON_MARKER: &str = "\nno-mistakes-diagnostics-json: ";

pub(crate) fn execute_json(
    input: &Buffer,
    run: impl FnOnce(Value) -> napi::Result<String>,
    text: bool,
) -> napi::Result<Buffer> {
    let (mut options, invocation_options) =
        no_mistakes::invocation::extract_napi_options_value(super::utf8_json(input)?)
            .map_err(to_napi_error)?;
    let request = DiagnosticRequestGuard::extract(&mut options).map_err(to_napi_error)?;
    let mut invocation_guard = None;
    let output = (|| {
        let prepare = || no_mistakes::invocation::InvocationGuard::acquire(invocation_options);
        invocation_guard = Some(
            match &request {
                Some(request) => {
                    request
                        .observer
                        .trace("invocation.prepare", TimingKind::Serial, prepare)
                }
                None => prepare(),
            }
            .map_err(to_invocation_napi_error)?,
        );
        let warning = no_mistakes::cli::init_rayon_threads_if_requested(invocation_options.jobs);
        no_mistakes::invocation::check_timeout().map_err(to_invocation_napi_error)?;
        let run = || no_mistakes::ast::with_request_parse_cache(|| run(options));
        let output = match &request {
            Some(request) => request
                .observer
                .trace("invocation.dispatch", TimingKind::Serial, run),
            None => run(),
        };
        no_mistakes::invocation::check_timeout().map_err(to_invocation_napi_error)?;
        if text {
            output
        } else {
            output.map(|json| no_mistakes::cli::attach_jobs_pool_warning(json, warning))
        }
    })();
    let output = finish(output, request.as_ref(), text);
    drop(invocation_guard);
    output
}

pub(crate) fn execute_pure(
    input: Value,
    run: impl FnOnce(Value) -> napi::Result<String>,
) -> napi::Result<Buffer> {
    let mut input = input;
    let request = DiagnosticRequestGuard::extract(&mut input).map_err(to_napi_error)?;
    let output = match &request {
        Some(request) => request
            .observer
            .trace("analysis", TimingKind::Serial, || run(input)),
        None => run(input),
    };
    finish(output, request.as_ref(), false)
}

fn finish(
    output: napi::Result<String>,
    request: Option<&DiagnosticRequestGuard>,
    text: bool,
) -> napi::Result<Buffer> {
    let Some(diagnostics) = request.and_then(|request| request.report(output.is_ok())) else {
        return output.map(|output| Buffer::from(output.into_bytes()));
    };
    let diagnostics = serde_json::json!(diagnostics).to_string();
    match output {
        // Native JSON stays opaque so source ASTs retain their full nesting depth.
        Ok(result) => {
            let result = if text {
                serde_json::Value::String(result).to_string()
            } else {
                result
            };
            Ok(Buffer::from(
                format!("{{\"result\":{result},\"diagnostics\":{diagnostics}}}").into_bytes(),
            ))
        }
        Err(mut error) => {
            error.reason.push_str(DIAGNOSTICS_JSON_MARKER);
            error.reason.push_str(&diagnostics);
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests;
