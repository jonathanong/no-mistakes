use crate::codebase::dependencies::graph::{
    DepGraph, EdgeKind, GraphBuildPlan, GraphFiles, NodeId, PreparedGraphBuild,
};
use crate::codebase::dependencies::{
    parse_entrypoint, relationship_filter, workflow_node_from_suffix, RelationshipArg,
};
use crate::codebase::ts_resolver::{
    find_tsconfig_from_visible, load_tsconfig, normalize_path, TsConfig,
};
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FlowOptions {
    pub target: String,
    pub root: PathBuf,
    pub tsconfig: Option<PathBuf>,
    pub config: Option<PathBuf>,
    pub direction: FlowDirection,
    pub depth: usize,
    pub relationships: Vec<RelationshipArg>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowDirection {
    Deps,
    Dependents,
    Both,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FlowReport {
    pub root: String,
    pub target: String,
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FlowNode {
    pub id: String,
    pub kind: &'static str,
    pub depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<usize>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    pub kind: &'static str,
}

pub fn run(options: &FlowOptions) -> Result<FlowReport> {
    let root = normalize_path(&options.root);
    let root = root.canonicalize().unwrap_or(root);
    let visible_paths = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let root_visible_paths = visible_paths.paths_for(&root);
    let graph_all_files =
        crate::codebase::ts_source::discover_files_from_visible(&root, &[], &root_visible_paths);
    let graph_files = GraphFiles::from_files_with_resource_candidates(
        graph_all_files,
        // Runtime resources may intentionally live under source-skipped
        // directories such as `fixtures/`. Reuse the request snapshot's
        // tracked inventory without another walk.
        visible_paths.tracked_paths_for(&root).as_ref().clone(),
    );
    let tsconfig =
        resolve_tsconfig_from_visible(&root, options.tsconfig.as_deref(), &root_visible_paths)?;
    let tsconfig_catalog = options.tsconfig.as_deref().map_or_else(
        || {
            crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
                &root,
                std::slice::from_ref(&root),
                &root_visible_paths,
            )
        },
        |path| {
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(path)
            };
            crate::codebase::ts_resolver::TsConfigCatalog::forced(
                &root,
                tsconfig.clone(),
                Some(normalize_path(&path)),
            )
        },
    );
    let allowed = relationship_filter(&options.relationships);
    let plan = GraphBuildPlan::from_allowed(allowed.as_ref()).with_symbols(true);
    let config = crate::config::v2::load_v2_config_from_visible(
        &root,
        options.config.as_deref(),
        &root_visible_paths,
    )?;
    let codebase_config =
        crate::codebase::config::config_from_loaded_v2(&root, options.config.as_deref(), &config);
    let prepared_graph = crate::codebase::dependencies::graph::prepare_graph_config(
        &root,
        plan,
        &codebase_config,
        &config,
        &visible_paths,
    )?;
    let graph = DepGraph::build_with_plan_files_prepared_config_facts_and_resolution_cache(
        PreparedGraphBuild {
            root: &root,
            tsconfig: &tsconfig,
            tsconfig_catalog: Some(&tsconfig_catalog),
            plan,
            graph_files: &graph_files,
            config_path: options.config.as_deref(),
            prepared: &prepared_graph,
            facts: None,
            import_resolution_cache: None,
            dotnet_facts: None,
            swift_facts: None,
            visible_paths: Some(&visible_paths),
        },
    )?;
    run_with_prepared_graph(options, &root, &graph)
}

pub(crate) fn run_with_prepared_graph(
    options: &FlowOptions,
    root: &Path,
    graph: &DepGraph,
) -> Result<FlowReport> {
    let target = resolve_target(root, &options.target);
    let allowed = relationship_filter(&options.relationships);
    let mut nodes = BTreeMap::new();
    let mut edges = BTreeSet::new();
    insert_node(&mut nodes, &target, root, 0);

    let mut traversal = Traversal {
        graph,
        root,
        max_depth: options.depth,
        allowed: allowed.as_ref(),
        nodes: &mut nodes,
        edges: &mut edges,
    };
    match options.direction {
        FlowDirection::Deps => traversal.traverse(&target, TraverseDirection::Deps)?,
        FlowDirection::Dependents => {
            traversal.traverse(&target, TraverseDirection::Dependents)?;
        }
        FlowDirection::Both => {
            traversal.traverse(&target, TraverseDirection::Deps)?;
            traversal.traverse(&target, TraverseDirection::Dependents)?;
        }
    }

    Ok(FlowReport {
        root: root.to_string_lossy().into_owned(),
        target: target.display_name(root).replace('\\', "/"),
        nodes: nodes.into_values().collect(),
        edges: edges.into_iter().collect(),
    })
}

include!("flow_query_traverse.rs");

#[cfg(test)]
#[path = "flow_query_tests.rs"]
mod flow_query_tests;

#[cfg(test)]
#[path = "flow_query_resource_tests.rs"]
mod flow_query_resource_tests;

#[cfg(test)]
#[path = "flow_query_timeout_tests.rs"]
mod flow_query_timeout_tests;

include!("flow_query_config.rs");
