pub use super::test_plan::{
    NamedFullSuiteTrigger, TestPlanConfig, TestPlanDependencies, TestPlanEnvironment,
    TestPlanFrameworkConfig, TestPlanGroup, TestPlanGroupType,
    TestPlanIgnoredChangedTestsFramework, TestPlanLimit, TestPlanPercent,
    TestPlanProjectDependency, TestPlanTargetedProjectDependency,
};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod applications;
mod ci_checks;
mod effects;
mod infra_config;
pub mod loaded_path;
pub(crate) mod postgres_executors;
mod project_postgres;
mod queues;
mod rule_targets;
mod string_or_list;
mod tests_config;

pub use queues::{
    EnqueueFactory, ProjectWaitScopes, QueueFactoryRoles, QueueNameFactory, WorkerFactory,
};

pub use ci_checks::{CheckCommandDef, CheckFileArgs, ChecksConfig, CiConfig};
pub use effects::{EffectKindConfig, EffectReturnedMemberConfig, EffectTargetConfig};
pub use infra_config::{InfraConfig, TerraformConfig, TerraformTestConvention};
pub use project_postgres::{
    ProjectPostgresConfig, ProjectPostgresExecutors, ProjectPostgresTrustedSqlTag,
};
pub use tests_config::{
    DartConfig, DotnetConfig, DotnetProjectConfig, ElixirConfig, GoConfig, ImpactConfig,
    JavaConfig, JestConfig, KotlinConfig, PhpConfig, PlaywrightAppBinding,
    PlaywrightCoverageConfig, PlaywrightSelectorWrapper, PlaywrightSelectors, PlaywrightTestConfig,
    PythonConfig, RailsConfig, RouteCoverageFramework, RouteCoverageHelper, RouteCoverageSource,
    RustLangConfig, StorybookConfig, SwiftConfig, TestProjectPolicy, Tests, VitestConfig,
};

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct NoMistakesConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<super::Effort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unanalyzable: Option<super::Unanalyzable>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unanalyzable_sql: Option<super::Unanalyzable>,
    #[serde(skip)]
    pub effort_override: Option<super::Effort>,
    #[serde(skip)]
    pub analysis_warnings: Vec<String>,
    #[serde(skip)]
    pub analysis_fallbacks: Vec<super::AnalysisFallback>,
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
    /// Set after PostgreSQL project scope inheritance. Not configuration.
    #[serde(skip, default)]
    pub postgres_project_scope_materialized: bool,
    /// Config file the loader read. Not part of the declared configuration.
    #[serde(skip, default)]
    pub loaded_config_path: loaded_path::LoadedConfigPath,
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

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct QueuesTopLevelConfig {
    #[serde(
        default,
        deserialize_with = "queues::deserialize_factories",
        serialize_with = "queues::serialize_factories"
    )]
    pub factories: QueueFactoryRoles,
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
    pub trpc: TrpcConfig,
    pub rewrites: Vec<RewriteRule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postgres: Option<ProjectPostgresConfig>,
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
    pub cluster: Option<String>,
    /// Files whose `addAndWait` / `waitUntilFinished` calls are checked.
    /// Empty means every scanned file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wait_scopes: Vec<String>,
}

include!("schema_trpc.rs");

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum StringOrList {
    One(String),
    Many(Vec<String>),
}

include!("schema/rules.rs");
