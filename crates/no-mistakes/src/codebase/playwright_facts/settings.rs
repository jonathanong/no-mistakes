use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub frontend_root: String,
    pub playwright_configs: Vec<PathBuf>,
    pub project: Option<String>,
    pub test_include: Vec<String>,
    pub test_exclude: Vec<String>,
    pub ignore_routes: Vec<String>,
    pub rewrites: Vec<crate::config::v2::schema::RewriteRule>,
    pub navigation_helpers: Vec<String>,
    pub route_coverage_sources: Vec<crate::config::v2::schema::RouteCoverageSource>,
    pub route_coverage_vitest: crate::config::v2::schema::VitestConfig,
    pub selector_wrappers: Vec<crate::config::v2::schema::PlaywrightSelectorWrapper>,
    pub selector_attributes: Vec<String>,
    /// Explicit override for the `getByTestId(...)` attribute, from
    /// `tests.playwright.testIdAttribute`. Use this when the Playwright config's
    /// `testIdAttribute` cannot be read statically (e.g. it is set inside a
    /// helper function).
    pub test_id_attribute_override: Option<String>,
    pub component_selector_attributes: BTreeMap<String, String>,
    pub html_ids: bool,
    pub selector_roots: Vec<String>,
    pub selector_include: Vec<String>,
    pub selector_exclude: Vec<String>,
}
