//! `lock status`: report the machine-wide invocation lock holder without waiting.
use super::options::{parse_options_value, to_napi_error};

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LockStatusOptions {}

pub(crate) fn lock_status_json_impl(options: serde_json::Value) -> napi::Result<String> {
    parse_options_value::<LockStatusOptions>(options)?;
    let report = crate::invocation::lock_status().map_err(to_napi_error)?;
    serde_json::to_string_pretty(&report)
        .map_err(|error| napi::Error::from_reason(error.to_string()))
}
