pub use crate::codebase::testing_analysis::runner::TestRunner;

pub use crate::codebase::testing_analysis::discovered_tests::DiscoveredTests;

/// Read-only identity of a runner project prepared for the current request.
/// It intentionally exposes only the values needed to validate target-scoped
/// test-plan triggers, not the runner-config implementation type.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedRunnerProject {
    pub config: Option<String>,
    pub runner_project_arg: Option<String>,
}
