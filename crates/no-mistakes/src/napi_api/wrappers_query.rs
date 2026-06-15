// Included into `napi_api` via `include!`; shares that module's imports.
// AsyncTask wrappers for the issue-419 query commands (stripped under coverage).

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "dataPwJson"))]
pub fn data_pw_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, data_pw_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "effectsJson"))]
pub fn effects_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, effects_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "rscCallersJson"))]
pub fn rsc_callers_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, rsc_callers_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "registryExtensionJson"))]
pub fn registry_extension_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, registry_extension_json_impl))
}
