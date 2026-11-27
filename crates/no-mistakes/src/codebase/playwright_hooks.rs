use super::analysis_session::PathInterner;
use super::check_facts::{
    CheckFactPlan, CheckFileFacts, PlaywrightFactPlan, PlaywrightSettingsKey, PlaywrightTestFacts,
    RunnerConfigFacts,
};
use super::dependencies::{
    extract::ImportedBinding,
    graph::{DepGraph, Edge, GraphBuildPlan, PlaywrightPreparationFailure, TsFactLookup},
};
use super::playwright_facts::{
    integration::{PreparedLinks, RouteOccurrence},
    selectors::AppSelector,
    text::AppTextTarget,
    Settings, StaticExportValues,
};
use super::ts_resolver::TsConfig;
use super::ts_source::{FileIdMap, SourceStore, VisiblePathSnapshot};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use dashmap::DashMap;
use globset::GlobSet;
use oxc_ast::ast::Program;
use oxc_semantic::SemanticBuilderReturn;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

/// Build selector dependencies after the initial graph exists so text-locator
/// reachability can reuse its RouteImport edges instead of constructing a
/// second graph.
pub(crate) struct PlaywrightSelectorEdgeInputs<'a> {
    pub(crate) all_files: &'a [PathBuf],
    pub(crate) facts: Option<&'a dyn TsFactLookup>,
    pub(crate) partial_graph: Option<&'a DepGraph>,
    pub(crate) graph_tsconfig: Option<&'a TsConfig>,
    pub(crate) snapshot: &'a VisiblePathSnapshot,
    pub(crate) prepared_settings: &'a [Settings],
    pub(crate) interner: &'a PathInterner,
}

impl PlaywrightSelectorEdgeInputs<'_> {
    pub(crate) fn route_import(&self) -> Option<(&DepGraph, &TsConfig)> {
        self.partial_graph.zip(self.graph_tsconfig)
    }
}

#[derive(Default)]
pub(crate) struct SourceFacts {
    pub(crate) selectors: HashMap<(PlaywrightSettingsKey, bool), Vec<AppSelector>>,
    pub(crate) text_targets: HashMap<PlaywrightSettingsKey, Vec<AppTextTarget>>,
    pub(crate) integration_routes: Vec<RouteOccurrence>,
}

pub(crate) type AppSelectorOccurrencesCache =
    DashMap<(PlaywrightSettingsKey, bool), Result<Arc<Vec<AppSelector>>, String>>;
pub(crate) type AppTextTargetsCache =
    DashMap<PlaywrightSettingsKey, Result<Arc<Vec<AppTextTarget>>, String>>;

type GraphSettings = fn(
    &Path,
    GraphBuildPlan,
    &NoMistakesConfig,
    &VisiblePathSnapshot,
    bool,
) -> Result<(Vec<Settings>, Vec<PlaywrightPreparationFailure>)>;
type GraphFactPlan =
    fn(&Path, &[Settings], &VisiblePathSnapshot) -> Result<Option<PlaywrightFactPlan>>;
type PartialGraphFactPlan = fn(
    &Path,
    &mut Vec<Settings>,
    &VisiblePathSnapshot,
    Vec<PlaywrightPreparationFailure>,
) -> (
    Option<PlaywrightFactPlan>,
    Vec<PlaywrightPreparationFailure>,
);
type RouteEdges = fn(
    &Path,
    Option<&Path>,
    &[PathBuf],
    Option<&dyn TsFactLookup>,
    &VisiblePathSnapshot,
    &[Settings],
    &PathInterner,
) -> Vec<Edge>;
type SelectorEdges =
    fn(&Path, Option<&Path>, PlaywrightSelectorEdgeInputs<'_>) -> Result<Vec<Edge>>;
type SelectorSourceFiles =
    fn(&Path, &Settings, &GlobSet, &GlobSet, bool, &VisiblePathSnapshot) -> Vec<PathBuf>;
type CollectSourceFacts =
    fn(&Path, &Path, &str, &Program<'_>, &PlaywrightFactPlan, &[ImportedBinding]) -> SourceFacts;
type CollectTestFacts = fn(
    &Path,
    &Program<'_>,
    &str,
    &PlaywrightFactPlan,
    &Option<SemanticBuilderReturn<'_>>,
) -> Option<PlaywrightTestFacts>;
type AggregateFacts =
    fn(&FileIdMap<CheckFileFacts>) -> (AppSelectorOccurrencesCache, AppTextTargetsCache);
type RunnerPlan = fn(&Path, &mut CheckFactPlan, &PlaywrightFactPlan, &Arc<SourceStore>);
type RouteLinks = fn(
    &Path,
    &CheckFactPlan,
    &PlaywrightFactPlan,
    &FileIdMap<CheckFileFacts>,
    RunnerConfigFacts,
) -> PreparedLinks;

pub(crate) struct Hooks {
    pub(crate) graph_settings: GraphSettings,
    pub(crate) graph_fact_plan: GraphFactPlan,
    pub(crate) partial_graph_fact_plan: PartialGraphFactPlan,
    pub(crate) route_edges: RouteEdges,
    pub(crate) selector_edges: SelectorEdges,
    pub(crate) selector_source_files: SelectorSourceFiles,
    pub(crate) source_facts: CollectSourceFacts,
    pub(crate) static_exports: fn(&Program<'_>) -> StaticExportValues,
    pub(crate) test_facts: CollectTestFacts,
    pub(crate) aggregate_facts: AggregateFacts,
    pub(crate) runner_plan: RunnerPlan,
    pub(crate) route_links: RouteLinks,
}

static HOOKS: OnceLock<&'static Hooks> = OnceLock::new();

pub(crate) fn register(hooks: &'static Hooks) {
    HOOKS.get_or_init(|| hooks);
}

pub(crate) fn hooks() -> &'static Hooks {
    HOOKS
        .get()
        .expect("Playwright analysis hooks registered by the facade")
}
