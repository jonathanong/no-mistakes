/// Compact Vitest project ownership retained until test-edge traversal.
pub(crate) struct VitestSetupProject {
    pub(crate) config: Option<String>,
    pub(crate) scope: Option<String>,
    pub(crate) filter: crate::codebase::testing_analysis::project_filter::ProjectTestFilter,
    /// Visible files explicitly matched by this project's Vitest globs. These
    /// may be non-TS/JS tests, so they do not necessarily have a base graph
    /// node before lazy setup edges are materialized.
    pub(crate) tests: Vec<PathBuf>,
    pub(crate) setups: Vec<(PathBuf, VitestSetupField)>,
}
