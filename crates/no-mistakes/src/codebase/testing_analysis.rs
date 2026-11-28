use crate::codebase::dependencies::extract::ExportedBinding;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId, TsFactLookup};
use crate::codebase::ts_resolver::TsConfig;
use crate::codebase::ts_source::{SourceStore, VisiblePathSnapshot};
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use oxc_ast::ast::Program;
use oxc_semantic::Semantic;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

mod catalog;
pub(crate) mod discovered_tests;
pub mod dynamic_imports;
pub mod matcher_properties;
pub mod plan;
mod prepared_projects;
pub mod project_filter;
pub(crate) mod project_patterns;
pub mod reliability;
pub mod runner;
pub mod runner_filter;
pub mod test_filter;
pub mod vitest_excludes;
pub mod vitest_policy;

pub(crate) use catalog::{FrameworkCatalog, FrameworkCatalogPreparation};
pub(crate) use prepared_projects::PreparedProjects;

type ConfigPaths =
    fn(&plan::FrameworkPreparationPlan, &Path, &NoMistakesConfig, &[PathBuf]) -> HashSet<PathBuf>;
type TestGlobs = fn(
    &Path,
    &str,
    &NoMistakesConfig,
    &TsConfig,
    &VisiblePathSnapshot,
    Option<&dyn PreparedProjects>,
) -> Vec<String>;
type FilterFromVisible = fn(&Path, &NoMistakesConfig, &[PathBuf]) -> test_filter::TestFileFilter;
type LegacyFilterFromVisible =
    fn(&Path, &NoMistakesConfig, &[PathBuf]) -> anyhow::Result<runner_filter::TestFilter>;
type CollectReliability = for<'a> fn(
    reliability::SourceDemand,
    &str,
    &'a Program<'a>,
    &Semantic<'a>,
) -> Vec<reliability::Location>;
type CollectFixtureInitializers =
    for<'a> fn(
        &'a Program<'a>,
        &Semantic<'a>,
        &[ExportedBinding],
    ) -> reliability::playwright_provenance::FixtureInitializers;
type CollectVitestExcludes =
    for<'a> fn(&'a Program<'a>, &str, &Semantic<'a>, bool) -> vitest_excludes::CollectedExcludes;
type VitestPolicyDiagnostics = fn(
    &Path,
    &NoMistakesConfig,
    &[PathBuf],
    &SourceStore,
    bool,
) -> anyhow::Result<vitest_policy::Diagnostics>;
type AssertionOwnershipEdges = fn(
    &Path,
    &[RuleDef],
    &DepGraph,
    &dyn TsFactLookup,
) -> anyhow::Result<Vec<(NodeId, NodeId, EdgeKind)>>;

pub struct Hooks {
    pub(crate) prepare_framework_catalog:
        fn(FrameworkCatalogPreparation<'_>) -> anyhow::Result<FrameworkCatalog>,
    pub excluded_config_paths: ConfigPaths,
    pub(crate) test_filters_from_prepared: TestGlobs,
    pub new_test_filter: fn(&Path, &NoMistakesConfig) -> test_filter::TestFileFilter,
    pub test_filter_from_visible: FilterFromVisible,
    pub legacy_filter_from_visible: LegacyFilterFromVisible,
    pub dynamic_imports_rule_id: &'static str,
    pub needs_playwright_configs: fn(&NoMistakesConfig) -> bool,
    pub needs_vitest_configs: fn(&NoMistakesConfig) -> bool,
    pub vitest_roots_configured: fn(&NoMistakesConfig) -> bool,
    pub(crate) collect_reliability: CollectReliability,
    pub(crate) collect_fixture_initializers: CollectFixtureInitializers,
    pub has_vitest_candidate_exports: fn(&Program<'_>) -> bool,
    pub(crate) collect_vitest_excludes: CollectVitestExcludes,
    pub vitest_policy_rule_id: &'static str,
    pub vitest_policy_diagnostics: VitestPolicyDiagnostics,
    pub assertion_owner_rule_id: &'static str,
    pub assertion_ownership_edges: AssertionOwnershipEdges,
}

static HOOKS: OnceLock<&'static Hooks> = OnceLock::new();

pub fn register(hooks: &'static Hooks) {
    HOOKS.get_or_init(|| hooks);
}

pub fn hooks() -> &'static Hooks {
    HOOKS
        .get()
        .copied()
        .expect("testing analysis hooks are registered at facade startup")
}
