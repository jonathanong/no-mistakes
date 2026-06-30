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
