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
        (self.run)(std::mem::take(&mut self.options_json))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

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
