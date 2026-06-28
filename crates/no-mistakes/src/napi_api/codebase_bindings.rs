#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "symbolsJson"))]
pub fn symbols_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, symbols_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "importUsagesJson"))]
pub fn import_usages_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, import_usages_json_impl))
}
