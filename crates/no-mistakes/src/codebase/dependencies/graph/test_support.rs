use super::*;
use std::collections::HashMap;
use std::path::PathBuf;

pub(super) fn test_interner() -> PathInterner {
    PathInterner::new()
}

mod edge_maps;
#[cfg(test)]
mod playwright;

pub(crate) use edge_maps::add_distinct_worker_file_edges;
pub(super) use edge_maps::add_queue_edges;
use edge_maps::edge_index_from_test_maps;
#[cfg(test)]
pub(super) use playwright::run_playwright_selector_analysis;

/// Construct a graph directly from pre-built maps for tests.
pub(crate) fn from_raw_maps(
    root: PathBuf,
    forward: HashMap<PathBuf, Vec<PathBuf>>,
    reverse: HashMap<PathBuf, Vec<PathBuf>>,
) -> DepGraph {
    let typed_fwd: EdgeMap = forward
        .into_iter()
        .map(|(k, vs)| {
            (
                NodeId::file(k),
                vs.into_iter()
                    .map(|v| (NodeId::file(v), EdgeKind::Import))
                    .collect(),
            )
        })
        .collect();
    let typed_rev: EdgeMap = reverse
        .into_iter()
        .map(|(k, vs)| {
            (
                NodeId::file(k),
                vs.into_iter()
                    .map(|v| (NodeId::file(v), EdgeKind::Import))
                    .collect(),
            )
        })
        .collect();
    DepGraph {
        root,
        edges: edge_index_from_test_maps(typed_fwd, typed_rev),
        vitest_setup_projects: Vec::new(),
        effective_edges: std::sync::OnceLock::new(),
        parse_errors: HashMap::new(),
        resource_edge_details: HashMap::new(),
        resource_diagnostics: Vec::new(),
    }
}

/// Construct a graph directly from typed edge maps for tests that need non-File nodes.
pub(crate) fn from_typed_maps(root: PathBuf, forward: EdgeMap, reverse: EdgeMap) -> DepGraph {
    DepGraph {
        root,
        edges: edge_index_from_test_maps(forward, reverse),
        vitest_setup_projects: Vec::new(),
        effective_edges: std::sync::OnceLock::new(),
        parse_errors: HashMap::new(),
        resource_edge_details: HashMap::new(),
        resource_diagnostics: Vec::new(),
    }
}

impl DepGraph {
    pub(crate) fn vitest_setup_edges_materialized(&self) -> bool {
        self.effective_edges.get().is_some()
    }

    pub(crate) fn build_with_plan_file_list_config_and_check_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        files: Vec<PathBuf>,
        config_path: Option<&Path>,
        facts: &crate::codebase::check_facts::CheckFactMap,
    ) -> Result<Self> {
        let graph_files = GraphFiles::from_files(files);
        Self::build_with_plan_files_config_and_facts(
            root,
            tsconfig,
            plan,
            &graph_files,
            config_path,
            Some(facts as &dyn TsFactLookup),
        )
    }

    pub(crate) fn build_with_plan_file_list_and_check_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        files: Vec<PathBuf>,
        facts: &crate::codebase::check_facts::CheckFactMap,
    ) -> Result<Self> {
        Self::build_with_plan_file_list_config_and_check_facts(
            root, tsconfig, plan, files, None, facts,
        )
    }

    pub(crate) fn build_with_plan_files_and_facts(
        root: &Path,
        tsconfig: &TsConfig,
        plan: GraphBuildPlan,
        graph_files: &GraphFiles,
        facts: Option<&dyn TsFactLookup>,
    ) -> Result<Self> {
        Self::build_with_plan_files_config_and_facts(root, tsconfig, plan, graph_files, None, facts)
    }
}

pub(super) fn collect_playwright_route_edges(
    root: &Path,
    config_path: Option<&Path>,
    all_files: &[PathBuf],
    facts: Option<&dyn TsFactLookup>,
) -> Vec<Edge> {
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(root);
    collect_playwright_route_edges_from_snapshot(
        root,
        config_path,
        all_files,
        facts,
        &snapshot,
        &[],
        &test_interner(),
    )
}

pub(super) fn collect_swift_edges(
    root: &Path,
    tsconfig: &TsConfig,
    all_files: &[PathBuf],
    config_options: Option<&GraphConfigOptions>,
) -> Vec<Edge> {
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let Some(config_options) = config_options else {
        return Vec::new();
    };
    collect_swift_edges_with_facts(
        SwiftEdgeInputs {
            root,
            tsconfig,
            tsconfig_catalog: None,
            all_files,
            config_options: Some(config_options),
            ts_facts: None,
            prepared_facts: None,
            session: &session,
        },
        session.interner(),
    )
}

pub(crate) fn ts_fact_context_for_plan(root: &Path, plan: GraphBuildPlan) -> TsFactContext {
    let options = graph_config_options_for_plan(root, plan);
    ts_fact_context_from_options(root, plan, options.as_ref())
}

pub(super) fn graph_config_options_for_plan(
    root: &Path,
    plan: GraphBuildPlan,
) -> Option<GraphConfigOptions> {
    if graph_plan_needs_config(plan) {
        graph_config_options(root)
    } else {
        None
    }
}
