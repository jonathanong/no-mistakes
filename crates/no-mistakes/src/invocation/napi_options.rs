use super::{nonzero_seconds, InvocationOptions, DEFAULT_TIMEOUT_SECONDS};
use anyhow::{anyhow, Context, Result};
use serde_json::{Map, Value};
use std::time::Duration;

/// Remove invocation controls before strict command-specific N-API option parsing.
pub fn extract_napi_options(options_json: String) -> Result<(String, InvocationOptions)> {
    let mut value: Value = serde_json::from_str(&options_json).context("invalid options JSON")?;
    let object = value
        .as_object_mut()
        .context("invalid options JSON: expected an object")?;
    let timeout = take_timeout(object, "timeout")?;
    let lock_timeout = take_timeout(object, "lockTimeout")?;
    let fail_on_lock = match object.remove("failOnLock") {
        None => false,
        Some(Value::Bool(value)) => value,
        Some(_) => {
            return Err(anyhow!(
                "invalid options JSON: failOnLock must be a boolean"
            ))
        }
    };
    Ok((
        serde_json::to_string(&value).context("serializing command options")?,
        InvocationOptions {
            timeout,
            lock_timeout,
            fail_on_lock,
        },
    ))
}

fn take_timeout(object: &mut Map<String, Value>, key: &str) -> Result<Option<Duration>> {
    match object.remove(key) {
        None => Ok(Some(Duration::from_secs(DEFAULT_TIMEOUT_SECONDS))),
        Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => {
            let seconds = value.as_u64().with_context(|| {
                format!("invalid options JSON: {key} must be a non-negative integer or null")
            })?;
            Ok(nonzero_seconds(seconds))
        }
        Some(_) => Err(anyhow!(
            "invalid options JSON: {key} must be a non-negative integer or null"
        )),
    }
}
