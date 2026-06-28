#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "testsPlanJson"))]
pub fn tests_plan_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, tests_plan_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "testsWhyJson"))]
pub fn tests_why_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, tests_why_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "testsCommentMarkdown"))]
pub fn tests_comment_markdown(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, tests_comment_markdown_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "testsGraphJson"))]
pub fn tests_graph_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, tests_graph_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "testsGraphMermaid"))]
pub fn tests_graph_mermaid(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, tests_graph_mermaid_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "testsImpactJson"))]
pub fn tests_impact_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, tests_impact_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "testsTargetsJson"))]
pub fn tests_targets_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, tests_targets_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "playwrightCheckJson"))]
pub fn playwright_check_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, playwright_check_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "playwrightEdgesJson"))]
pub fn playwright_edges_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, playwright_edges_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "playwrightRelatedJson"))]
pub fn playwright_related_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, playwright_related_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "playwrightTestsJson"))]
pub fn playwright_tests_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, playwright_tests_json_impl))
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
#[cfg_attr(not(test), napi(js_name = "serverContractsJson"))]
pub fn server_contracts_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, server_contracts_json_impl))
}

#[cfg(not(coverage))]
#[cfg_attr(not(test), napi(js_name = "flowJson"))]
pub fn flow_json(options_json: String) -> AsyncTask<JsonTask> {
    AsyncTask::new(JsonTask::new(options_json, flow_json_impl))
}
