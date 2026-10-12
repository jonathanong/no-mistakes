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
    let options: Options =
        serde_json::from_value(options).map_err(super::async_task::to_napi_error)?;
    serde_json::to_string(&crate::diagnostics::progress_for_request(&options.id))
        .map_err(super::async_task::to_napi_error)
}

#[cfg(test)]
mod tests;
