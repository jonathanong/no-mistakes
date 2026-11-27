use crate::integration_tests::types::Framework;

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

pub(super) fn framework(runner: TestRunner) -> Framework {
    match runner {
        TestRunner::Dotnet => Framework::Dotnet,
        TestRunner::Playwright => Framework::Playwright,
        TestRunner::Vitest => Framework::Vitest,
        TestRunner::Swift => Framework::Swift,
        TestRunner::Python => Framework::Python,
        TestRunner::Go => Framework::Go,
        TestRunner::Cargo => Framework::Cargo,
        TestRunner::Rails => Framework::Rails,
        TestRunner::Php => Framework::Php,
        TestRunner::Java => Framework::Java,
        TestRunner::Kotlin => Framework::Kotlin,
        TestRunner::Elixir => Framework::Elixir,
        TestRunner::Dart => Framework::Dart,
        TestRunner::Jest => Framework::Jest,
    }
}
