#[cfg(not(coverage))]
use super::async_task;
#[cfg(all(not(test), not(coverage)))]
use napi_derive::napi;

json_binding!(
    diagnostics_snapshot_json,
    "diagnosticsSnapshotJson",
    super::diagnostics_snapshot_impl,
    independent
);

pub(crate) fn diagnostics_snapshot_impl(options: serde_json::Value) -> napi::Result<String> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Options {
        id: String,
    }
    let options: Options = serde_json::from_value(options)
        .map_err(|error| napi::Error::from_reason(error.to_string()))?;
    serde_json::to_string(&crate::diagnostics::progress_for_request(&options.id))
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}

#[cfg(test)]
mod tests;
