use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub total_routes: usize,
    pub covered_routes: usize,
    pub uncovered_routes: usize,
    pub total_selectors: usize,
    pub covered_selectors: usize,
    pub uncovered_selectors: usize,
    pub duplicate_selectors: usize,
    pub total_fetch_apis: usize,
    pub covered_fetch_apis: usize,
    pub uncovered_fetch_apis: usize,
}
