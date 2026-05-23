#![cfg_attr(all(coverage, not(test)), allow(dead_code, unused_imports))]

#[cfg(not(coverage))]
use napi::bindgen_prelude::AsyncTask;
#[cfg(all(not(test), not(coverage)))]
use napi_derive::napi;

mod async_task;
mod codebase;
mod options;
mod project;

#[cfg(not(coverage))]
use async_task::{JsonTask, VersionTask};
pub(crate) use codebase::{
    dependencies_json_impl, dependents_json_impl, related_json_impl, symbols_json_impl,
};
pub(crate) use project::{
    queue_check_json_impl, queue_edges_json_impl, queue_related_json_impl, queues_json_impl,
    react_analyze_json_impl, react_check_json_impl, server_route_edges_json_impl,
    server_route_list_json_impl, server_route_related_json_impl, server_routes_json_impl,
};

#[cfg(test)]
mod tests;

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi)]
pub fn version() -> AsyncTask<VersionTask> {
    AsyncTask::new(VersionTask)
}

pub(crate) fn version_impl() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "dependenciesJson"))]
pub fn dependencies_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, dependencies_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "dependentsJson"))]
pub fn dependents_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, dependents_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "relatedJson"))]
pub fn related_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, related_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "symbolsJson"))]
pub fn symbols_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, symbols_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "queuesJson"))]
pub fn queues_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, queues_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "queueEdgesJson"))]
pub fn queue_edges_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, queue_edges_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "queueRelatedJson"))]
pub fn queue_related_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, queue_related_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "queueCheckJson"))]
pub fn queue_check_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, queue_check_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "serverRoutesJson"))]
pub fn server_routes_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, server_routes_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "serverRouteListJson"))]
pub fn server_route_list_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, server_route_list_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "serverRouteEdgesJson"))]
pub fn server_route_edges_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, server_route_edges_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "serverRouteRelatedJson"))]
pub fn server_route_related_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, server_route_related_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "reactAnalyzeJson"))]
pub fn react_analyze_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, react_analyze_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "reactCheckJson"))]
pub fn react_check_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, react_check_json_impl))
}
