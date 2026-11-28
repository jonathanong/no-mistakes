pub(crate) use crate::codebase::playwright_facts::text::SelectorRef;
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;

pub(crate) use super::helper_references::{
    SelectorHelperReference, SelectorHelperReferenceWithValue, TestFileAnalysis,
};
use crate::playwright::config::Settings;
use crate::playwright::routes::Route;
use crate::playwright::selectors;

#[path = "types/summary.rs"]
mod summary;
pub use summary::Summary;

#[path = "types/test_ref.rs"]
mod test_ref;
pub use test_ref::{RouteCoverageAttribution, TestRef};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageRoute {
    pub route: String,
    pub file: String,
    pub covered: bool,
    pub tests: Vec<String>,
    pub tests_detail: Vec<TestRef>,
    pub urls: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageSelector {
    pub attribute: String,
    pub value: String,
    pub file: String,
    pub covered: bool,
    pub unsupported_dynamic: bool,
    pub tests: Vec<String>,
    pub tests_detail: Vec<TestRef>,
    pub selectors: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) helper_references: Vec<SelectorHelperReference>,
    #[serde(skip)]
    pub line: usize,
    #[serde(skip)]
    pub literal_exact: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageFetch {
    pub method: String,
    pub path: String,
    pub covered: bool,
    pub tests: Vec<String>,
    pub tests_detail: Vec<TestRef>,
    pub route_files: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateSelector {
    pub attribute: String,
    pub value: String,
    pub file: String,
    pub line: usize,
}

mod coverage_report;
use coverage_report::is_arc_empty;
pub use coverage_report::CoverageReport;

#[derive(Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Edge {
    #[serde(rename_all = "camelCase")]
    Fetch {
        test_file: Arc<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        test_name: Option<Arc<String>>,
        #[serde(skip_serializing_if = "is_arc_empty", default)]
        describe_path: Arc<Vec<String>>,
        route_file: Arc<String>,
        route: Arc<String>,
        method: String,
        path: String,
        side: String,
        cached: bool,
    },
    #[serde(rename_all = "camelCase")]
    Route {
        test_file: Arc<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        test_name: Option<Arc<String>>,
        #[serde(skip_serializing_if = "is_arc_empty", default)]
        describe_path: Arc<Vec<String>>,
        route_file: Arc<String>,
        route: Arc<String>,
        url: Arc<String>,
        hook: bool,
        line: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        attribution: Option<RouteCoverageAttribution>,
    },
    #[serde(rename_all = "camelCase")]
    Selector {
        test_file: Arc<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        test_name: Option<Arc<String>>,
        #[serde(skip_serializing_if = "is_arc_empty", default)]
        describe_path: Arc<Vec<String>>,
        app_file: Arc<String>,
        attribute: String,
        value: String,
        selector: String,
        line: u32,
    },
    #[serde(rename_all = "camelCase")]
    LocatorText {
        test_file: Arc<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        test_name: Option<Arc<String>>,
        #[serde(skip_serializing_if = "is_arc_empty", default)]
        describe_path: Arc<Vec<String>>,
        app_file: Arc<String>,
        locator_kind: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        role: Option<String>,
        text: String,
        locator: String,
        #[serde(skip_serializing)]
        test_id_attributes: Vec<String>,
        selector_refs: Vec<SelectorRef>,
        reasons: Vec<String>,
        line: u32,
    },
}

#[derive(Serialize)]
pub struct EdgeReport {
    pub edges: Vec<Edge>,
}

#[derive(Serialize)]
pub struct RelatedReport {
    pub tests: Vec<String>,
    pub fetch_apis: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    pub coverage: CoverageReport,
    pub edges: EdgeReport,
    /// Diagnostic-only records have root-relative paths and never enter coverage output.
    #[serde(skip)]
    pub unanalyzable_html_ids: Vec<selectors::AppSelector>,
}

pub struct CoverageInputs<'a> {
    pub root: &'a Path,
    pub routes: &'a [Route],
    pub app_selectors: &'a [selectors::AppSelector],
    pub app_selector_occurrences: &'a [selectors::AppSelector],
    pub edges: &'a [Edge],
    pub(crate) helper_references: &'a [SelectorHelperReferenceWithValue],
    pub settings: &'a Settings,
    pub unique_selector_policy: UniqueSelectorPolicy,
    pub fetch_index: &'a FetchIndex,
}

#[derive(Clone, Copy, Default)]
pub struct UniqueSelectorPolicy {
    pub test_ids: bool,
    pub html_ids: bool,
    pub aggregate: bool,
    pub configured_html_id_selector: bool,
    /// Skip recovered Exact values when judging unique test IDs.
    pub literal_test_ids_only: bool,
    /// HTML-ID effort is independent of configured test-ID effort.
    pub literal_html_ids_only: bool,
}

pub type FetchIndex = std::collections::HashMap<String, Vec<crate::fetch::types::FetchOccurrence>>;
pub type SelectorCoverageKey = (Arc<String>, String, String);
pub type CoverageLinks = (
    std::collections::BTreeSet<Arc<String>>,
    std::collections::BTreeSet<String>,
);
