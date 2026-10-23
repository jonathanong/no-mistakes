pub(super) mod deadline_evidence;
pub use deadline_evidence::*;
mod deadlines;
pub(crate) use deadlines::{
    DeadlineDeclaration, DeadlineInheritance, DeadlineUnknownReason, DeadlineValue,
    DeclaredDeadlines,
};

mod vitest_setup;
pub(crate) use vitest_setup::{VitestSetupDependency, VitestSetupField};

use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationFinding {
    pub framework: String,
    pub suite: String,
    pub file: String,
    pub line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub describe_path: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integration: Option<String>,
    pub message: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Framework {
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
    Dart,
    Jest,
}

impl Framework {
    pub(crate) fn as_str(self) -> &'static str {
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
            Self::Dart => "dart",
            Self::Jest => "jest",
        }
    }

    pub(crate) fn has_js_runner_config(self) -> bool {
        matches!(self, Self::Playwright | Self::Vitest | Self::Jest)
    }
}
#[derive(Debug, Clone)]
pub(super) struct Suite {
    pub framework: Framework,
    pub name: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub policy: EffectiveIntegrationPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum EffectiveIntegrationPolicy {
    AllowedIntegrations { integrations: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TestCase {
    pub name: Option<String>,
    pub describe_path: Vec<String>,
    pub function_key: FunctionKey,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct FunctionKey {
    pub file: PathBuf,
    pub name: String,
}

#[derive(Debug, Clone)]
pub(super) struct FunctionInfo {
    pub integration: Option<String>,
    pub calls: Vec<CallTarget>,
}

#[derive(Debug, Clone)]
pub(super) enum CallTarget {
    Local(String),
    Imported { local: String },
    Namespace { namespace: String, member: String },
}

#[derive(Debug, Clone)]
pub(super) struct ImportBinding {
    pub source: String,
    pub imported: ImportedName,
}

#[derive(Debug, Clone)]
pub(super) enum ImportedName {
    Named(String),
    Default,
    Namespace,
}

#[derive(Clone, Default)]
pub(crate) struct FileAnalysis {
    pub(super) imports: HashMap<String, ImportBinding>,
    pub(super) exports: HashMap<String, String>,
    pub(super) functions: HashMap<String, FunctionInfo>,
    pub(super) tests: Vec<TestCase>,
}

#[derive(Debug, Clone)]
pub(crate) struct ConfigProject {
    pub(crate) config: Option<String>,
    /// The Vitest runner source is a workspace/project-array file and must be
    /// passed with `--workspace` rather than `--config`.
    pub(crate) workspace: bool,
    pub(crate) policy_name: Option<String>,
    pub(crate) runner_project_arg: Option<String>,
    /// Relative-to-root directory this project globs (its testDir / project
    /// root). `None` for explicit-policy projects, which are never dominated by
    /// config-scoped ownership filtering.
    pub(crate) scope: Option<String>,
    pub(crate) include: Vec<String>,
    pub(crate) exclude: Vec<String>,
    /// Vitest setup modules statically declared for this effective project.
    /// Other runners always leave this empty.
    pub(crate) vitest_setup: Vec<VitestSetupDependency>,
    pub(crate) declared_deadlines: DeclaredDeadlines,
}
