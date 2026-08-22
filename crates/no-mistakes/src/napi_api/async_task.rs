use napi::bindgen_prelude::Buffer;
use napi::{Env, Task};

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
        ensure_rayon_threads();
        let options_json = utf8_json(&self.options_json)?;
        let (options, invocation_options) =
            crate::invocation::extract_napi_options_value(options_json).map_err(to_napi_error)?;
        let _guard = crate::invocation::InvocationGuard::acquire(invocation_options)
            .map_err(to_napi_error)?;
        crate::cli::init_rayon_threads_if_requested(invocation_options.jobs);
        crate::invocation::check_timeout().map_err(to_napi_error)?;
        let output = crate::ast::with_request_parse_cache(|| (self.run)(options));
        crate::invocation::check_timeout().map_err(to_napi_error)?;
        output.map(Buffer::from)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

impl JsonTask {
    pub(crate) fn new(options_json: Buffer, run: fn(String) -> napi::Result<String>) -> Self {
        Self { options_json, run }
    }
}

impl Task for JsonTask {
    type Output = Buffer;
    type JsValue = Buffer;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        ensure_rayon_threads();
        let options_json = utf8_json(&self.options_json)?;
        let (options_json, invocation_options) =
            crate::invocation::extract_napi_options(options_json).map_err(to_napi_error)?;
        let _guard = crate::invocation::InvocationGuard::acquire(invocation_options)
            .map_err(to_napi_error)?;
        crate::cli::init_rayon_threads_if_requested(invocation_options.jobs);
        crate::invocation::check_timeout().map_err(to_napi_error)?;
        let output = crate::ast::with_request_parse_cache(|| (self.run)(options_json));
        crate::invocation::check_timeout().map_err(to_napi_error)?;
        output.map(Buffer::from)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

fn utf8_json(options_json: &Buffer) -> napi::Result<String> {
    String::from_utf8(options_json.to_vec())
        .map_err(|error| napi::Error::from_reason(format!("options JSON must be UTF-8: {error}")))
}

fn ensure_rayon_threads() {
    crate::cli::init_rayon_threads(crate::cli::JobsArg { jobs: 0 });
}

fn to_napi_error(error: anyhow::Error) -> napi::Error {
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
