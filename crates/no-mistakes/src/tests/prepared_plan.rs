use super::changed_files::ChangedFiles;
use super::dotnet_dependency_changes::DotnetDependencyAnalysis;
use super::lockfile_changes::LockfileAnalysis;
use super::package_manifest_changes::PackageManifestAnalysis;
use super::swift_manifest_changes::SwiftManifestAnalysis;
use super::swift_resolved_changes::SwiftResolvedAnalysis;
use super::{PlanArgs, TestFramework};
use anyhow::Result;
use no_mistakes::codebase::dependencies::graph::{
    DepGraph, GraphBuildPlan, GraphFiles, PreparedGraphConfig,
};
use no_mistakes::codebase::test_discovery::{DiscoveredTests, TestRunner};
use no_mistakes::codebase::ts_resolver::TsConfig;
use no_mistakes::codebase::ts_source::{SourceStore, VisiblePathSnapshot};
use no_mistakes::codebase::workspaces::WorkspaceMap;
use no_mistakes::config::v2::NoMistakesConfig;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex, OnceLock};

mod graph_inputs;
mod helpers;
mod inputs;
mod request;
use helpers::{test_framework, test_runner};
pub(crate) mod revisions;

/// Immutable, request-scoped inputs shared by direct test plans and the
/// multi-framework impacted-check fanout.
pub(crate) struct PreparedTestPlanInputs {
    args: PlanArgs,
    pub(crate) root: PathBuf,
    pub(crate) visible_paths: Arc<VisiblePathSnapshot>,
    root_visible_paths: Arc<Vec<PathBuf>>,
    pub(crate) config: NoMistakesConfig,
    config_path: Option<PathBuf>,
    pub(crate) collected: ChangedFiles,
}

pub(crate) struct PreparedTestPlanRequest {
    args: PlanArgs,
    pub(crate) root: PathBuf,
    pub(crate) visible_paths: Arc<VisiblePathSnapshot>,
    sources: Arc<SourceStore>,
    root_visible_paths: Arc<Vec<PathBuf>>,
    pub(crate) config: NoMistakesConfig,
    config_path: Option<PathBuf>,
    pub(crate) tsconfig: TsConfig,
    tsconfig_catalog: Arc<no_mistakes::codebase::ts_resolver::TsConfigCatalog>,
    pub(crate) collected: ChangedFiles,
    pub(crate) changed_files: Vec<PathBuf>,
    pub(crate) package_manifest_analysis: PackageManifestAnalysis,
    pub(crate) lockfile_analysis: LockfileAnalysis,
    pub(crate) swift_resolved_analysis: SwiftResolvedAnalysis,
    pub(crate) swift_manifest_analysis: SwiftManifestAnalysis,
    pub(crate) dotnet_dependency_analysis: DotnetDependencyAnalysis,
    pub(crate) lockfile_changed_packages: Vec<(String, String, Vec<PathBuf>)>,
    pub(crate) workspace_map: WorkspaceMap,
    graph_files: GraphFiles,
    graph_plan: GraphBuildPlan,
    prepared_graph_config: PreparedGraphConfig,
    prepared_test_projects: Arc<no_mistakes::codebase::test_discovery::PreparedTestProjects>,
    test_filter: no_mistakes::codebase::test_filter::TestFileFilter,
    graph: OnceLock<std::result::Result<DepGraph, String>>,
    discovered_tests: Mutex<HashMap<TestFramework, std::result::Result<DiscoveredTests, String>>>,
    changes_input: OnceLock<Option<super::plan::changes::ChangesInput>>,
    config_invalidation: OnceLock<
        std::result::Result<Option<super::config_invalidation::ConfigInvalidation>, String>,
    >,
    graph_builds: AtomicUsize,
    framework_discoveries: AtomicUsize,
}

pub(crate) fn javascript_dependency_framework(framework: Option<TestFramework>) -> bool {
    matches!(
        framework,
        None | Some(TestFramework::Vitest)
            | Some(TestFramework::Playwright)
            | Some(TestFramework::Jest)
    )
}

#[path = "prepared_plan_discovery.rs"]
mod prepared_plan_discovery;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "prepared_plan/tests.rs"]
mod tests;
