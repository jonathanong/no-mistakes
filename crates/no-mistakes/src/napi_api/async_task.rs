use napi::{Env, Task};

pub struct JsonTask {
    options_json: String,
    run: fn(String) -> napi::Result<String>,
}

impl JsonTask {
    pub(crate) fn new(options_json: String, run: fn(String) -> napi::Result<String>) -> Self {
        Self { options_json, run }
    }
}

impl Task for JsonTask {
    type Output = String;
    type JsValue = String;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let options_json = std::mem::take(&mut self.options_json);
        let (options_json, invocation_options) =
            crate::invocation::extract_napi_options(options_json).map_err(to_napi_error)?;
        let _guard = crate::invocation::InvocationGuard::acquire(invocation_options)
            .map_err(to_napi_error)?;
        crate::invocation::check_timeout().map_err(to_napi_error)?;
        let output = crate::ast::with_request_parse_cache(|| (self.run)(options_json));
        crate::invocation::check_timeout().map_err(to_napi_error)?;
        output
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
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
