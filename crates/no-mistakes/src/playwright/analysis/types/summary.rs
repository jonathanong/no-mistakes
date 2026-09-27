use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Summary {
    pub(crate) total_routes: usize,
    pub(crate) covered_routes: usize,
    pub(crate) uncovered_routes: usize,
    pub(crate) total_selectors: usize,
    pub(crate) covered_selectors: usize,
    pub(crate) uncovered_selectors: usize,
    pub(crate) duplicate_selectors: usize,
    pub(crate) total_fetch_apis: usize,
    pub(crate) covered_fetch_apis: usize,
    pub(crate) uncovered_fetch_apis: usize,
}
