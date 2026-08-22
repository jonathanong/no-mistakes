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
    Python,
    Go,
    Cargo,
    Rails,
    Php,
    Java,
    Kotlin,
    Elixir,
    Jest,
}

impl TestRunner {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "dotnet" => Some(Self::Dotnet),
            "playwright" => Some(Self::Playwright),
            "vitest" => Some(Self::Vitest),
            "swift" => Some(Self::Swift),
            "python" => Some(Self::Python),
            "go" => Some(Self::Go),
            "cargo" => Some(Self::Cargo),
            "rails" => Some(Self::Rails),
            "php" => Some(Self::Php),
            "java" => Some(Self::Java),
            "kotlin" => Some(Self::Kotlin),
            "elixir" => Some(Self::Elixir),
            "jest" => Some(Self::Jest),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dotnet => "dotnet",
            Self::Playwright => "playwright",
            Self::Vitest => "vitest",
            Self::Swift => "swift",
            Self::Python => "python",
            Self::Go => "go",
            Self::Cargo => "cargo",
            Self::Rails => "rails",
            Self::Php => "php",
            Self::Java => "java",
            Self::Kotlin => "kotlin",
            Self::Elixir => "elixir",
            Self::Jest => "jest",
        }
    }

    pub(super) fn is_language_frontend(self) -> bool {
        matches!(
            self,
            Self::Python
                | Self::Go
                | Self::Cargo
                | Self::Rails
                | Self::Php
                | Self::Java
                | Self::Kotlin
                | Self::Elixir
        )
    }

    pub(super) fn framework(self) -> Framework {
        match self {
            Self::Dotnet => Framework::Dotnet,
            Self::Playwright => Framework::Playwright,
            Self::Vitest => Framework::Vitest,
            Self::Swift => Framework::Swift,
            Self::Python => Framework::Python,
            Self::Go => Framework::Go,
            Self::Cargo => Framework::Cargo,
            Self::Rails => Framework::Rails,
            Self::Php => Framework::Php,
            Self::Java => Framework::Java,
            Self::Kotlin => Framework::Kotlin,
            Self::Elixir => Framework::Elixir,
            Self::Jest => Framework::Jest,
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
