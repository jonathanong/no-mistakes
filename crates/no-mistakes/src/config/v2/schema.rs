pub use super::test_plan::{
    TestPlanConfig, TestPlanDependencies, TestPlanEnvironment, TestPlanFrameworkConfig,
    TestPlanGroup, TestPlanGroupType, TestPlanIgnoredChangedTestsFramework, TestPlanLimit,
    TestPlanPercent, TestPlanProjectDependency, TestPlanTargetedProjectDependency,
};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod ci_checks;
mod infra_config;
mod rule_targets;
mod string_or_list;
mod tests_config;

pub use ci_checks::{CheckCommandDef, CheckFileArgs, ChecksConfig, CiConfig};
pub use infra_config::{InfraConfig, TerraformConfig, TerraformTestConvention};
pub use tests_config::{
    DotnetConfig, DotnetProjectConfig, ImpactConfig, JestConfig, PlaywrightSelectorWrapper,
    PlaywrightSelectors, PlaywrightTestConfig, StorybookConfig, SwiftConfig, TestProjectPolicy,
    Tests, VitestConfig,
};

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct NoMistakesConfig {
    /// Legacy React analyzer root retained for compatibility with the
    /// standalone `react` commands and the aggregate `check` command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontend_root: Option<String>,
    /// Legacy top-level React no-fetch assertion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assert_no_fetch: Option<bool>,
    /// Legacy nested React analyzer settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub react_traits: Option<ReactTraitsConfig>,
    pub filesystem: FilesystemConfig,
    pub infra: InfraConfig,
    pub projects: BTreeMap<String, Project>,
    pub queues: QueuesTopLevelConfig,
    /// Named effect families for the `effects` query, keyed by `<kind>`.
    pub effects: BTreeMap<String, EffectKindConfig>,
    pub tests: Tests,
    #[serde(rename = "testPlan", alias = "test_plan")]
    pub test_plan: TestPlanConfig,
    pub rules: Vec<RuleDef>,
    /// GitHub Actions workflow analysis configuration (`no-mistakes ci`).
    pub ci: CiConfig,
    /// Changed-file validation command mappings (`no-mistakes impacted-checks`).
    pub checks: ChecksConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ReactTraitsConfig {
    pub frontend_root: Option<String>,
    pub assert_no_fetch: Option<bool>,
}

/// One named effect family (e.g. `valkey`) for the `effects` query.
///
/// `categories` maps a category label (e.g. `cache`, `pubsub`) to the function
/// or constructor names that belong to it; `functions` is a flat list applied
/// when no category split is needed (reported as uncategorized).
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct EffectKindConfig {
    pub categories: BTreeMap<String, Vec<String>>,
    pub functions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct QueuesTopLevelConfig {
    pub factories: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct FilesystemConfig {
    pub skip_directories: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Project {
    #[serde(rename = "type")]
    pub type_: Option<ProjectType>,
    pub root: Option<String>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub routes: Vec<String>,
    pub queues: QueueConfig,
    pub rewrites: Vec<RewriteRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RewriteRule {
    pub source: String,
    pub destination: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectType {
    Server,
    Nextjs,
    Remix,
    Vitejs,
    Library,
    Tests,
    Rust,
    CloudflareWorkers,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct QueueConfig {
    pub enqueues: Vec<String>,
    pub workers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum StringOrList {
    One(String),
    Many(Vec<String>),
}

/// A configured rule application.
///
/// Unlike ESLint-style rule maps, no-mistakes rules are reusable applications:
/// the same `rule` can be attached to different projects or test groups with
/// different names and options.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct RuleDef {
    pub name: Option<String>,
    pub rule: String,
    pub message: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub projects: Vec<String>,
    pub tests: RuleTestTargets,
    pub scope: Option<RuleScope>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    #[serde(default = "empty_options")]
    pub options: serde_yaml::Value,
}

impl Default for RuleDef {
    fn default() -> Self {
        Self {
            name: None,
            rule: String::new(),
            message: None,
            enabled: true,
            projects: Vec::new(),
            tests: RuleTestTargets::default(),
            scope: None,
            include: Vec::new(),
            exclude: Vec::new(),
            options: empty_options(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct RuleTestTargets {
    pub dotnet: Vec<String>,
    pub vitest: Vec<String>,
    pub playwright: Vec<String>,
    pub swift: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum RuleScope {
    Repository,
}

fn default_true() -> bool {
    true
}

fn empty_options() -> serde_yaml::Value {
    serde_yaml::Value::Mapping(Default::default())
}

impl RuleDef {
    pub fn rule_options<T: for<'de> serde::Deserialize<'de> + Default>(&self) -> T {
        serde_yaml::from_value(self.options.clone()).unwrap_or_default()
    }

    pub fn applies_to_project(&self, project: &str) -> bool {
        self.enabled && self.projects.iter().any(|name| name == project)
    }

    pub fn applies_to_repository(&self) -> bool {
        self.enabled && self.scope == Some(RuleScope::Repository)
    }
}

impl NoMistakesConfig {
    pub fn rule_applications<'a>(&'a self, rule_id: &str) -> Vec<&'a RuleDef> {
        self.rules
            .iter()
            .filter(move |rule| {
                rule.enabled && rule.rule == rule_id && self.rule_has_effective_target(rule)
            })
            .collect()
    }

    pub fn rule_configured(&self, rule_id: &str) -> bool {
        !self.rule_applications(rule_id).is_empty()
    }

    pub fn rule_application_options<T: for<'de> serde::Deserialize<'de> + Default>(
        &self,
        rule_id: &str,
    ) -> Vec<T> {
        self.rule_applications(rule_id)
            .into_iter()
            .map(RuleDef::rule_options)
            .collect()
    }

    fn rule_has_effective_target(&self, rule: &RuleDef) -> bool {
        rule.scope == Some(RuleScope::Repository)
            || rule
                .projects
                .iter()
                .any(|project| self.projects.contains_key(project))
            || rule_targets::rule_has_effective_test_target(rule)
    }
}
