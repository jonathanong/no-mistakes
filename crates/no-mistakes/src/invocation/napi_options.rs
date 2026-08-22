use super::{nonzero_seconds, InvocationArgs};
use anyhow::{anyhow, Context, Result};
use serde_json::{Map, Value};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvocationOptions {
    pub timeout: Option<Duration>,
    pub lock_timeout: Option<Duration>,
    pub fail_on_lock: bool,
    /// Rayon worker count. `None` leaves the pool unchanged; `Some(0)` uses
    /// the CPU count, matching CLI `--jobs 0`.
    pub jobs: Option<usize>,
}

impl Default for InvocationOptions {
    fn default() -> Self {
        InvocationArgs::default().options()
    }
}

/// Remove invocation controls before strict command-specific N-API option parsing.
pub fn extract_napi_options(options_json: impl AsRef<str>) -> Result<(String, InvocationOptions)> {
    let (value, options) = extract_napi_options_value(options_json)?;
    Ok((
        serde_json::to_string(&value).context("serializing command options")?,
        options,
    ))
}

/// Parse N-API invocation controls while retaining the command options as a
/// structured value for entrypoints that can avoid a second JSON parse.
pub fn extract_napi_options_value(
    options_json: impl AsRef<str>,
) -> Result<(Value, InvocationOptions)> {
    let mut value: Value =
        serde_json::from_str(options_json.as_ref()).context("invalid options JSON")?;
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
    let jobs = take_jobs(object)?;
    Ok((
        value,
        InvocationOptions {
            timeout,
            lock_timeout,
            fail_on_lock,
            jobs,
        },
    ))
}

fn take_jobs(object: &mut Map<String, Value>) -> Result<Option<usize>> {
    match object.remove("jobs") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(value)) => value
            .as_u64()
            .context("invalid options JSON: jobs must be a non-negative integer or null")
            .map(|jobs| Some(jobs as usize)),
        Some(_) => Err(anyhow!(
            "invalid options JSON: jobs must be a non-negative integer or null"
        )),
    }
}

fn take_timeout(object: &mut Map<String, Value>, key: &str) -> Result<Option<Duration>> {
    match object.remove(key) {
        None | Some(Value::Null) => Ok(None),
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
