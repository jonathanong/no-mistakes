use serde::Serialize;

pub(crate) use no_mistakes::fetch::types::{CacheKind, FetchOccurrence, FetchSide, SourceType};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RouteReport {
    pub(crate) route: String,
    pub(crate) file: String,
    pub(crate) api_calls: Vec<FetchOccurrence>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FinalReport {
    pub(crate) summary: Summary,
    pub(crate) routes: Vec<RouteReport>,
    pub(crate) duplicates: Vec<DuplicateApiCall>,
    pub(crate) unsupported: Vec<UnsupportedApiCall>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Summary {
    pub(crate) total_routes: usize,
    pub(crate) routes_with_api_calls: usize,
    pub(crate) total_api_calls: usize,
    pub(crate) unique_api_calls: usize,
    pub(crate) duplicate_api_calls: usize,
    pub(crate) dynamic_api_calls: usize,
    pub(crate) cached_api_calls: usize,
    pub(crate) client_api_calls: usize,
    pub(crate) server_api_calls: usize,
    pub(crate) rsc_api_calls: usize,
    pub(crate) conditional_api_calls: usize,
    pub(crate) parallel_api_calls: usize,
    pub(crate) error_handled_api_calls: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DuplicateApiCall {
    pub(crate) key: String,
    pub(crate) count: usize,
    pub(crate) occurrences: Vec<ApiCallOccurrence>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApiCallOccurrence {
    pub(crate) route: String,
    pub(crate) file: String,
    pub(crate) line: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UnsupportedApiCall {
    pub(crate) route: String,
    pub(crate) file: String,
    pub(crate) line: usize,
    pub(crate) reason: String,
    pub(crate) raw_path: String,
}
