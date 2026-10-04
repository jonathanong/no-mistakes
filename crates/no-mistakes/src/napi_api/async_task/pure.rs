//! Source-only capabilities do not acquire a filesystem invocation lock.
use napi::{bindgen_prelude::Buffer, Env, Task};

pub struct PureJsonTask {
    input: Buffer,
    run: fn(serde_json::Value) -> napi::Result<String>,
}

impl PureJsonTask {
    pub(crate) fn new(input: Buffer, run: fn(serde_json::Value) -> napi::Result<String>) -> Self {
        Self { input, run }
    }
}

impl Task for PureJsonTask {
    type Output = Buffer;
    type JsValue = Buffer;
    fn compute(&mut self) -> napi::Result<Buffer> {
        let input = serde_json::from_str(super::utf8_json(&self.input)?)
            .map_err(|error| napi::Error::from_reason(error.to_string()))?;
        (self.run)(input).map(|output| Buffer::from(output.into_bytes()))
    }
    fn resolve(&mut self, _: Env, output: Buffer) -> napi::Result<Buffer> {
        Ok(output)
    }
}

#[cfg(test)]
mod tests;
