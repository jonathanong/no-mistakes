use super::{Options, RULE_ID};
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::codebase::dependencies::{relationship_filter, EdgeKind};
use crate::codebase::ts_resolver::normalize_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Historical config-only plan, including the broad default when no inventory
/// is available. Check entrypoints use the fallible inventory-aware variant.
pub fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    let applications = config.rule_applications(RULE_ID);
    if applications.is_empty() {
        return None;
    }
    let opts_list: Vec<Options> = applications
        .iter()
        .map(|rule| rule.rule_options())
        .collect();
    let union_allowed = union_allowed_set(&opts_list);
    Some(GraphBuildPlan::from_allowed(union_allowed.as_ref()))
}

/// Fallible config-only plan for callers without a prepared file inventory.
pub fn try_graph_plan(config: &NoMistakesConfig) -> Result<Option<GraphBuildPlan>> {
    try_graph_plan_with_files(None, config, &[], false)
}

/// Only the unfiltered default contribution can omit families proved empty by
/// the complete visible inventory. Explicit relationship selections stay exact.
pub(crate) fn try_graph_plan_for_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Option<GraphBuildPlan>> {
    try_graph_plan_with_files(Some(root), config, all_files, true)
}

pub(crate) fn try_graph_plan_for_files_inventory_only(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Option<GraphBuildPlan>> {
    try_graph_plan_with_files(Some(root), config, all_files, false)
}

fn try_graph_plan_with_files(
    root: Option<&Path>,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    config_matches_graph: bool,
) -> Result<Option<GraphBuildPlan>> {
    let applications = config.rule_applications(RULE_ID);
    if applications.is_empty() {
        return Ok(None);
    }
    let opts_list: Vec<Options> = applications
        .iter()
        .map(|rule| rule.try_rule_options())
        .collect::<Result<_>>()?;
    Ok(Some(match root {
        Some(root) => graph_plan_for_options_with_graph_config(
            root,
            config,
            all_files,
            &opts_list,
            config_matches_graph,
        ),
        None => GraphBuildPlan::from_allowed(union_allowed_set(&opts_list).as_ref()),
    }))
}

pub(super) fn graph_plan_for_options_with_graph_config(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    opts_list: &[Options],
    config_matches_graph: bool,
) -> GraphBuildPlan {
    let mut plan = GraphBuildPlan::default();
    let mut default_plan = None;
    for opts in opts_list {
        if opts.relationships.is_empty() {
            let default = default_plan.get_or_insert_with(|| {
                let allowed = relationship_filter(&opts.relationships);
                let mut default = GraphBuildPlan::from_allowed(allowed.as_ref());
                prune_default_families(&mut default, root, config, all_files, config_matches_graph);
                default
            });
            plan.include(*default);
        } else {
            let allowed = relationship_filter(&opts.relationships);
            plan.include(GraphBuildPlan::from_allowed(allowed.as_ref()));
        }
    }
    plan
}

pub(crate) fn union_allowed_set(opts_list: &[Options]) -> Option<HashSet<EdgeKind>> {
    let mut any_all = false;
    let mut union: HashSet<EdgeKind> = HashSet::new();
    for opts in opts_list {
        match relationship_filter(&opts.relationships) {
            Some(set) => union.extend(set),
            None => any_all = true,
        }
    }
    if any_all || union.is_empty() {
        None
    } else {
        Some(union)
    }
}

include!("plan/inventory.rs");
