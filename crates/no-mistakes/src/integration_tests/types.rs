use serde::Serialize;
use std::collections::{BTreeSet, HashMap};
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
}

impl Framework {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Dotnet => "dotnet",
            Self::Playwright => "playwright",
            Self::Vitest => "vitest",
            Self::Swift => "swift",
        }
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
}

/// The Vitest config field that declared a setup module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum VitestSetupField {
    SetupFiles,
    GlobalSetup,
}

impl VitestSetupField {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::SetupFiles => "setupFiles",
            Self::GlobalSetup => "globalSetup",
        }
    }
}

/// A single statically declared (or conservatively dynamic) Vitest setup
/// dependency. `specifier` is absent only for a dynamic expression; a literal
/// with no `resolved_path` is an unresolved candidate and must remain visible
/// to impact planning.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct VitestSetupDependency {
    pub(crate) field: VitestSetupField,
    pub(crate) specifier: Option<String>,
    /// An imported setup helper could not resolve through the preliminary
    /// catalog, but its bare specifier may resolve through a package catalog
    /// discovered from runner project roots.
    pub(crate) needs_final_catalog_reparse: bool,
    /// A static config `extends` target could not be read, so this synthetic
    /// dependency keeps a project-bounded fallback trigger without inventing
    /// a setup module edge.
    pub(crate) unresolved_config_extends: Option<String>,
    /// A readable static config `extends` target. This preserves its resolved
    /// path and candidates for owner-scoped config-change planning without
    /// creating a synthetic setup edge.
    pub(crate) config_extends_provenance: bool,
    pub(crate) resolved_path: Option<PathBuf>,
    /// Effective Vitest project root used to resolve a relative setup
    /// specifier. This deliberately differs from `declaration_path` for
    /// imported config objects and projects with an explicit `root`.
    pub(crate) resolution_base: PathBuf,
    pub(crate) declaration_path: PathBuf,
    pub(crate) declaration_line: u32,
    /// Config, helper, and literal-resolution candidate modules that
    /// contributed this declaration. These paths are conservative impact
    /// triggers, including targets deleted before planning.
    pub(crate) trigger_paths: BTreeSet<PathBuf>,
    /// Candidates derived from the import resolver. Keep these separate from
    /// parser provenance so a later scoped re-resolution can replace only its
    /// own facts without dropping config and helper triggers.
    pub(crate) resolver_candidate_paths: BTreeSet<PathBuf>,
    /// Runtime candidates reached through a resolved setup module. These stay
    /// separate so a deletion can recover the missing graph edge without
    /// treating unrelated alternate candidates as unsafe fallbacks.
    pub(crate) transitive_trigger_paths: BTreeSet<PathBuf>,
}
