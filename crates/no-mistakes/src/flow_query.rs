use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, GraphBuildPlan, NodeId};
use crate::codebase::dependencies::{
    parse_entrypoint, relationship_filter, workflow_node_from_suffix, RelationshipArg,
};
use crate::codebase::ts_resolver::normalize_path;
use anyhow::Result;
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
    let allowed = relationship_filter(&options.relationships);
    let plan = GraphBuildPlan::from_allowed(allowed.as_ref()).with_symbols(true);
    let mut framework_plan =
        crate::codebase::test_discovery::FrameworkPreparationPlan::for_graph(plan);
    if let Some(path) = resolve_target(&root, &options.target).as_file() {
        framework_plan.retain_indexable_path(path.to_path_buf());
    }
    let mut traversal =
        crate::codebase::dependencies::SharedTraversalContext::prepare_with_framework_plan(
            root,
            options.tsconfig.as_deref(),
            options.config.as_deref(),
            plan,
            framework_plan,
        )?;
    traversal.flow_report(options)
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
#[path = "flow_query_vitest_tests.rs"]
mod flow_query_vitest_tests;

#[cfg(test)]
#[path = "flow_query_resource_tests.rs"]
mod flow_query_resource_tests;

#[cfg(test)]
#[path = "flow_query_timeout_tests.rs"]
mod flow_query_timeout_tests;
