use super::{check_rule_application, union_allowed_set, Options, RULE_ID};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use std::path::Path;

pub(super) fn validate_shared_graph_plan(
    root: &Path,
    config_path: Option<&Path>,
    shared: &crate::codebase::check_facts::CheckFactMap,
    prepared_graph: Option<&crate::codebase::dependencies::graph::PreparedGraphConfig>,
    plan: GraphBuildPlan,
) -> Result<()> {
    let (required_graph_plan, _) = match prepared_graph {
        Some(prepared) => {
            crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
                root, plan, prepared,
            )
        }
        None => {
            crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_config(
                root,
                plan,
                config_path,
            )
        }
    };
    if !shared.graph_plan().covers(required_graph_plan) {
        bail!(
            "shared check facts are missing graph facts required by {RULE_ID}; collect facts with forbidden_dependencies::graph_plan before calling run_check_with_facts"
        );
    }
    Ok(())
}

pub(crate) fn check_with_prepared_facts_and_graph(
    root: &Path,
    config: &NoMistakesConfig,
    config_path: Option<&Path>,
    shared: &crate::codebase::check_facts::CheckFactMap,
    prepared_graph: Option<&crate::codebase::dependencies::graph::PreparedGraphConfig>,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
    graph: &DepGraph,
) -> Result<Vec<RuleFinding>> {
    let applications = config.rule_applications(RULE_ID);
    if applications.is_empty() {
        return Ok(Vec::new());
    }
    let opts_list: Vec<Options> = applications
        .iter()
        .map(|rule| rule.rule_options())
        .collect();
    let plan = GraphBuildPlan::from_allowed(union_allowed_set(&opts_list).as_ref());
    validate_shared_graph_plan(root, config_path, shared, prepared_graph, plan)?;
    let file_universe = shared
        .graph_file_universe()
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<std::collections::HashSet<_>>();
    let mut findings = Vec::new();
    for (rule, opts) in applications.iter().zip(opts_list.iter()) {
        findings.extend(check_rule_application(
            root,
            config,
            rule,
            opts,
            graph,
            inferred_roots,
            Some(&file_universe),
        )?);
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}
