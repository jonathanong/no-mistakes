use crate::integration_tests::types::Framework;
use std::collections::BTreeMap;
use std::path::PathBuf;

use super::targets::TestExecutionTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TestRunner {
    Dotnet,
    Playwright,
    Vitest,
    Swift,
}

impl TestRunner {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "dotnet" => Some(Self::Dotnet),
            "playwright" => Some(Self::Playwright),
            "vitest" => Some(Self::Vitest),
            "swift" => Some(Self::Swift),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dotnet => "dotnet",
            Self::Playwright => "playwright",
            Self::Vitest => "vitest",
            Self::Swift => "swift",
        }
    }

    pub(super) fn framework(self) -> Framework {
        match self {
            Self::Dotnet => Framework::Dotnet,
            Self::Playwright => Framework::Playwright,
            Self::Vitest => Framework::Vitest,
            Self::Swift => Framework::Swift,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredTests {
    pub tests: Vec<PathBuf>,
    pub targets_by_path: BTreeMap<PathBuf, Vec<TestExecutionTarget>>,
    pub used_fallback: bool,
}

/// Read-only identity of a runner project prepared for the current request.
/// It intentionally exposes only the values needed to validate target-scoped
/// test-plan triggers, not the runner-config implementation type.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedRunnerProject {
    pub config: Option<String>,
    pub runner_project_arg: Option<String>,
}
