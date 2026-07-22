use super::check_with_prepared_facts_graph_and_session;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::ts_resolver::{TsConfig, TsConfigCatalog};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

pub(crate) fn check_with_prepared_facts_and_session(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig: &TsConfig,
    tsconfig_catalog: &TsConfigCatalog,
    shared: &CheckFactMap,
    session: &std::sync::Arc<crate::codebase::analysis_session::AnalysisSession>,
) -> Result<Vec<super::RuleFinding>> {
    let graph = crate::perf_trace::trace("test_no_unmocked_dynamic_imports.graph_build", || {
        DepGraph::build_with_complete_check_facts_and_session(
            crate::codebase::dependencies::graph::CompleteCheckFactGraphBuildRequest {
                root,
                tsconfig,
                tsconfig_catalog,
                plan: GraphBuildPlan::imports_and_workspace(),
                files: shared.graph_file_universe().to_vec(),
                config_path: None,
                facts: shared,
            },
            session.clone(),
        )
    })?;
    check_with_prepared_facts_graph_and_session(
        root,
        config,
        tsconfig,
        tsconfig_catalog,
        shared,
        &graph,
        session,
    )
}
