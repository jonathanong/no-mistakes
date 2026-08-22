use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{RewriteRule, StringOrList};

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Tests {
    pub playwright: PlaywrightTestConfig,
    pub vitest: VitestConfig,
    pub dotnet: DotnetConfig,
    pub swift: SwiftConfig,
    pub python: PythonConfig,
    pub go: GoConfig,
    pub rust: RustLangConfig,
    pub rails: RailsConfig,
    pub php: PhpConfig,
    pub java: JavaConfig,
    pub jest: JestConfig,
    pub storybook: StorybookConfig,
    pub impact: ImpactConfig,
}

/// Opt-in knobs for the `tests impact` query. Both lists default to empty, so
/// without configuration `tests impact` behaves exactly as before.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ImpactConfig {
    /// Glob patterns for stub/mock test files (e.g. `**/*.mock.test.*`) that
    /// `tests impact` must always surface when they transitively import a
    /// changed file, even when a configured test-suite `exclude` glob would
    /// otherwise drop them from test discovery.
    pub always_include_tests: Vec<String>,
    /// Glob patterns for "registry" files (e.g. `**/auth-gated-code-splitting.mts`,
    /// `**/*-registry.mts`). When a changed file is imported by a file matching one
    /// of these globs, `tests impact` emits a hint to verify the registry entry.
    pub registries: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct PlaywrightTestConfig {
    pub configs: Option<StringOrList>,
    pub projects: BTreeMap<String, TestProjectPolicy>,
    pub selectors: PlaywrightSelectors,
    /// Explicit `getByTestId(...)` attribute. Set this when the Playwright
    /// config's `testIdAttribute` is not statically readable (for example it is
    /// assigned inside a config helper function), so selector coverage can match
    /// `getByTestId` assertions against the right attribute. Serialized as
    /// `testIdAttribute`.
    pub test_id_attribute: Option<String>,
    pub test_include: Vec<String>,
    pub test_exclude: Vec<String>,
    pub selector_roots: Vec<String>,
    pub selector_include: Vec<String>,
    pub selector_exclude: Vec<String>,
    pub navigation_helpers: Vec<String>,
    pub frontend_root: Option<String>,
    pub ignore_routes: Option<Vec<String>>,
    /// Explicit per-Playwright-project frontend app bindings, keyed by the
    /// Playwright project name (the same name used in `rules[].tests.playwright`
    /// and in `projects`/`selectors` above). Distinct from `projects`, which
    /// scopes *test files*: `apps` answers "which frontend app does this
    /// Playwright project exercise" so `playwright-coverage` and
    /// `playwright-unique-test-ids` resolve routes/selectors against the right
    /// app when a repository configures more than one `type: nextjs` project.
    /// A Playwright project binds to at most one frontend app.
    pub apps: BTreeMap<String, PlaywrightAppBinding>,
}

/// Explicit frontend-app binding for one Playwright project. Every field is
/// optional and overrides the value [`crate::config::v2::frontend_apps`]
/// would otherwise resolve for the bound app.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct PlaywrightAppBinding {
    /// The `.no-mistakes.yml` `projects:` key of the frontend app this
    /// Playwright project exercises. Required when the repository configures
    /// more than one `type: nextjs` project and this Playwright project is
    /// not otherwise bound via a rule's `projects:` list.
    pub project: Option<String>,
    pub frontend_root: Option<String>,
    pub selector_roots: Vec<String>,
    pub rewrites: Vec<RewriteRule>,
    pub ignore_routes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct PlaywrightSelectors {
    pub html_ids: bool,
    pub test_ids: Vec<String>,
    pub component_test_ids: BTreeMap<String, String>,
    pub wrappers: Vec<PlaywrightSelectorWrapper>,
}

/// A statically imported helper whose configured argument carries the same
/// selector value as Playwright's `getByTestId(...)`.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaywrightSelectorWrapper {
    pub module: String,
    pub export: String,
    pub test_id_argument: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct VitestConfig {
    pub configs: Option<StringOrList>,
    pub projects: BTreeMap<String, TestProjectPolicy>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct SwiftConfig {
    pub packages: Vec<String>,
    pub projects: BTreeMap<String, TestProjectPolicy>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct PythonConfig {
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct GoConfig {
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct RustLangConfig {
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct RailsConfig {
    pub apps: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct PhpConfig {
    pub framework: Option<String>,
    pub apps: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct JavaConfig {
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct DotnetConfig {
    pub solutions: Vec<String>,
    pub projects: BTreeMap<String, DotnetProjectConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DotnetProjectConfig {
    pub project: String,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub test: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct JestConfig {
    pub configs: Option<StringOrList>,
    pub projects: BTreeMap<String, TestProjectPolicy>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct StorybookConfig {
    pub configs: Option<StringOrList>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TestProjectPolicy {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    #[serde(rename = "integration_suites")]
    pub integration_suites: BTreeMap<String, Vec<String>>,
}
