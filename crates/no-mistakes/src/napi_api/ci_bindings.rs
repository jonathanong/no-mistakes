// N-API bindings for the `ci` and `impacted-checks` commands. Included by
// napi_api.rs so the `#[napi]` registrations live in the crate-root module.

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "ciImpactJson"))]
pub fn ci_impact_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, ci_impact_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "ciEnvJson"))]
pub fn ci_env_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, ci_env_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "ciTopologyJson"))]
pub fn ci_topology_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, ci_topology_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "impactedChecksJson"))]
pub fn impacted_checks_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, impacted_checks_json_impl))
}
