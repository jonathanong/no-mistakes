use napi::bindgen_prelude::Buffer;
use napi::{Env, Task};
pub(crate) mod independent;
mod pure;
#[cfg(not(coverage))]
pub use pure::PureJsonTask;

#[cfg_attr(not(test), allow(dead_code))]
pub struct JsonTask {
    options_json: Buffer,
    run: fn(String) -> napi::Result<String>,
}

pub struct JsonValueTask {
    options_json: Buffer,
    run: fn(serde_json::Value) -> napi::Result<String>,
}

impl JsonValueTask {
    pub(crate) fn new(
        options_json: Buffer,
        run: fn(serde_json::Value) -> napi::Result<String>,
    ) -> Self {
        Self { options_json, run }
    }
}

impl Task for JsonValueTask {
    type Output = Buffer;
    type JsValue = Buffer;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let options_json = utf8_json(&self.options_json)?;
        let (mut options, invocation_options) =
            crate::invocation::extract_napi_options_value(options_json).map_err(to_napi_error)?;
        let _diagnostics = crate::diagnostics::DiagnosticRequestGuard::extract(&mut options)
            .map_err(to_napi_error)?;
        let prepare = || crate::invocation::InvocationGuard::acquire(invocation_options);
        let _guard = match &_diagnostics {
            Some(request) => request.observer.trace(
                "invocation.prepare",
                crate::diagnostics::TimingKind::Serial,
                prepare,
            ),
            None => prepare(),
        }
        .map_err(to_napi_error)?;
        crate::cli::init_rayon_threads_if_requested(invocation_options.jobs);
        crate::invocation::check_timeout().map_err(to_napi_error)?;
        let run = || crate::ast::with_request_parse_cache(|| (self.run)(options));
        let output = match &_diagnostics {
            Some(request) => request.observer.trace(
                "invocation.dispatch",
                crate::diagnostics::TimingKind::Serial,
                run,
            ),
            None => run(),
        };
        crate::invocation::check_timeout().map_err(to_napi_error)?;
        output.map(|json| Buffer::from(json.into_bytes()))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
impl JsonTask {
    pub(crate) fn new(options_json: Buffer, run: fn(String) -> napi::Result<String>) -> Self {
        Self { options_json, run }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
impl Task for JsonTask {
    type Output = Buffer;
    type JsValue = Buffer;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let options_json = utf8_json(&self.options_json)?;
        let (mut options, invocation_options) =
            crate::invocation::extract_napi_options_value(options_json).map_err(to_napi_error)?;
        let _diagnostics = crate::diagnostics::DiagnosticRequestGuard::extract(&mut options)
            .map_err(to_napi_error)?;
        let options_json = serde_json::to_string(&options)
            .map_err(|error| napi::Error::from_reason(error.to_string()))?;
        let prepare = || crate::invocation::InvocationGuard::acquire(invocation_options);
        let _guard = match &_diagnostics {
            Some(request) => request.observer.trace(
                "invocation.prepare",
                crate::diagnostics::TimingKind::Serial,
                prepare,
            ),
            None => prepare(),
        }
        .map_err(to_napi_error)?;
        crate::cli::init_rayon_threads_if_requested(invocation_options.jobs);
        crate::invocation::check_timeout().map_err(to_napi_error)?;
        let run = || crate::ast::with_request_parse_cache(|| (self.run)(options_json));
        let output = match &_diagnostics {
            Some(request) => request.observer.trace(
                "invocation.dispatch",
                crate::diagnostics::TimingKind::Serial,
                run,
            ),
            None => run(),
        };
        crate::invocation::check_timeout().map_err(to_napi_error)?;
        output.map(|json| Buffer::from(json.into_bytes()))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

fn utf8_json(options_json: &Buffer) -> napi::Result<&str> {
    std::str::from_utf8(options_json.as_ref())
        .map_err(|error| napi::Error::from_reason(format!("options JSON must be UTF-8: {error}")))
}

pub(super) fn to_napi_error(error: impl std::fmt::Display) -> napi::Error {
    napi::Error::from_reason(format!("{error:#}"))
}

#[cfg(test)]
mod tests;

pub struct VersionTask;

impl Task for VersionTask {
    type Output = String;
    type JsValue = String;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        Ok(super::version_impl())
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}
