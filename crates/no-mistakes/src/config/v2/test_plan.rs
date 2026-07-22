use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TestPlanConfig {
    pub dotnet: TestPlanFrameworkConfig,
    pub playwright: TestPlanFrameworkConfig,
    pub vitest: TestPlanFrameworkConfig,
    pub swift: TestPlanFrameworkConfig,
}

/// Raw deserialization target that accepts both `fullSuiteTriggers` (current)
/// and the deprecated `dependencies` key.
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
struct TestPlanFrameworkConfigRaw {
    /// Current name.
    full_suite_triggers: Option<TestPlanDependencies>,
    /// Deprecated alias – still accepted but emits a warning at load time.
    dependencies: Option<TestPlanDependencies>,
    environments: BTreeMap<String, TestPlanEnvironment>,
}

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TestPlanFrameworkConfig {
    pub full_suite_triggers: TestPlanDependencies,
    /// Set to `true` when the config file used the deprecated `dependencies` key.
    #[serde(skip)]
    pub deprecated_dependencies_key: bool,
    pub environments: BTreeMap<String, TestPlanEnvironment>,
}

impl<'de> Deserialize<'de> for TestPlanFrameworkConfig {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = TestPlanFrameworkConfigRaw::deserialize(deserializer)?;
        let (full_suite_triggers, deprecated) = match (raw.full_suite_triggers, raw.dependencies) {
            (Some(fst), _) => (fst, false),
            (None, Some(deps)) => (deps, true),
            (None, None) => (TestPlanDependencies::default(), false),
        };
        Ok(TestPlanFrameworkConfig {
            full_suite_triggers,
            deprecated_dependencies_key: deprecated,
            environments: raw.environments,
        })
    }
}

/// Backward-compatible alias so existing call-sites that read `.dependencies`
/// still compile. Prefer `.full_suite_triggers` in new code.
impl TestPlanFrameworkConfig {
    #[inline]
    pub fn dependencies(&self) -> &TestPlanDependencies {
        &self.full_suite_triggers
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TestPlanDependencies {
    #[serde(alias = "ignore_changed_tests")]
    pub ignore_changed_tests: Vec<TestPlanIgnoredChangedTestsFramework>,
    pub projects: BTreeMap<String, TestPlanProjectDependency>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TestPlanIgnoredChangedTestsFramework {
    Dotnet,
    Playwright,
    Vitest,
    Swift,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum TestPlanProjectDependency {
    All(bool),
    Patterns(Vec<String>),
    Targeted(TestPlanTargetedProjectDependency),
}

/// A path trigger that selects tests for only the named runner projects.
///
/// The map key that contains this value still identifies a no-mistakes
/// resource project. `targets` deliberately contains runner project names
/// (for example Vitest `--project` names), not resource-project names.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestPlanTargetedProjectDependency {
    pub paths: Vec<String>,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TestPlanEnvironment {
    pub all: bool,
    #[serde(alias = "global_config_fallback")]
    pub global_config_fallback: Option<bool>,
    pub limit: Option<TestPlanLimit>,
    pub groups: Vec<TestPlanGroup>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TestPlanGroup {
    #[serde(rename = "type")]
    pub type_: TestPlanGroupType,
    pub limit: Option<TestPlanLimit>,
    #[serde(alias = "sample_when_limited")]
    pub sample_when_limited: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TestPlanGroupType {
    #[default]
    Direct,
    Coverage,
    Dependencies,
    Sample,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct TestPlanLimit {
    pub percent: Option<TestPlanPercent>,
    pub files: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum TestPlanPercent {
    Number(f64),
    String(String),
}

impl TestPlanPercent {
    pub fn value(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            Self::String(raw) => raw.trim().trim_end_matches('%').parse().ok(),
        }
    }
}
