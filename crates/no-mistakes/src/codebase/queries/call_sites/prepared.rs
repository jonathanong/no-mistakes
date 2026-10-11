use super::super::shared::{ReversePrepared, Target};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, PreparedGraphBuild};
use crate::codebase::ts_source::facts::TsFactPlan;

pub(super) struct CallSitesAnalysis {
    pub(super) facts: crate::codebase::ts_source::facts::TsFactMap,
    pub(super) graph: DepGraph,
    pub(super) files: crate::codebase::dependencies::graph::GraphFiles,
}

pub(super) fn prepare(target: &Target) -> anyhow::Result<CallSitesAnalysis> {
    let prepared = target.prepare_reverse()?;
    let facts = super::super::reverse::collect_prepared_reverse_facts(
        target,
        &prepared,
        TsFactPlan {
            function_calls: true,
            call_sites: true,
            ..Default::default()
        },
    );
    let graph = build_graph(
        target,
        &prepared.graph_files,
        &prepared.tsconfig_catalog,
        prepared.workspace.clone(),
        &facts,
    )?;
    Ok(CallSitesAnalysis {
        facts,
        graph,
        files: prepared.graph_files,
    })
}

fn build_graph(
    target: &Target,
    graph_files: &crate::codebase::dependencies::graph::GraphFiles,
    catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
    workspace: std::sync::Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
    facts: &crate::codebase::ts_source::facts::TsFactMap,
) -> anyhow::Result<DepGraph> {
    let plan = GraphBuildPlan {
        calls: true,
        ..GraphBuildPlan::imports_and_workspace()
    };
    // Lightweight call queries do not load domain configuration.
    let config = crate::config::v2::NoMistakesConfig::default();
    let codebase_config =
        crate::codebase::config::config_from_loaded_v2(&target.root, None, &config);
    let graph_config =
        crate::codebase::dependencies::graph::prepare_graph_config_with_test_filter_and_workspace(
            &target.root,
            plan,
            &codebase_config,
            &config,
            &target.visible_paths,
            crate::codebase::test_filter::TestFileFilter::fallback_only(),
            workspace,
        )?;
    let graph = DepGraph::build_with_plan_files_prepared_config_facts_resolution_cache_and_session(
        PreparedGraphBuild {
            root: &target.root,
            tsconfig: catalog.config_for(&target.abs_file),
            tsconfig_catalog: Some(catalog),
            plan,
            graph_files,
            config_path: None,
            prepared: &graph_config,
            facts: Some(facts),
            import_resolution_cache: None,
            dotnet_facts: None,
            swift_facts: None,
            visible_paths: Some(&target.visible_paths),
        },
        target.session.clone(),
    )?;
    Ok(graph)
}

mod ordinary;
pub(crate) use ordinary::{OrdinaryCallSites, OrdinaryCallSitesPlan};
