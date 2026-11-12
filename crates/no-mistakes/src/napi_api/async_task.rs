use napi::bindgen_prelude::Buffer;
use napi::{Env, Task};
pub(crate) mod independent;
mod observed;
mod pure;
#[cfg(not(coverage))]
pub use pure::PureJsonTask;

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "Tests retain the string-task adapter.")
)]
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
        observed::execute_json(&self.options_json, self.run)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "Tests construct the string-task adapter.")
)]
impl JsonTask {
    pub(crate) fn new(options_json: Buffer, run: fn(String) -> napi::Result<String>) -> Self {
        Self { options_json, run }
    }
}

impl Task for JsonTask {
    type Output = Buffer;
    type JsValue = Buffer;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        observed::execute_json(&self.options_json, |options| {
            (self.run)(serde_json::to_string(&options).map_err(to_napi_error)?)
        })
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

fn utf8_json(options_json: &Buffer) -> napi::Result<&str> {
    std::str::from_utf8(options_json.as_ref())
        .map_err(|error| napi::Error::from_reason(format!("options JSON must be UTF-8: {error}")))
}

/// Marker that introduces the machine-readable lock/deadline error appended to
/// the message. The JS facade strips it and exposes `error.code` / `error.details`.
pub(crate) const ERROR_JSON_MARKER: &str = "\nno-mistakes-error-json: ";

pub(super) fn to_invocation_napi_error(error: anyhow::Error) -> napi::Error {
    let mut reason = format!("{error:#}");
    if let Some(report) = crate::invocation::error_report(&error) {
        reason.push_str(ERROR_JSON_MARKER);
        reason.push_str(&report["error"].to_string());
    }
    napi::Error::from_reason(reason)
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
