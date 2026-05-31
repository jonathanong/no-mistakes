use super::{check_rule_application, resolve_tsconfig, union_allowed_set, Options, RULE_ID};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use std::path::Path;

pub(crate) fn check_with_facts(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig_path: Option<&Path>,
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let applications = config.rule_applications(RULE_ID);
    if applications.is_empty() {
        return Ok(Vec::new());
    }
    let opts_list: Vec<Options> = applications.iter().map(|r| r.rule_options()).collect();
    let union_allowed = union_allowed_set(&opts_list);
    let plan = GraphBuildPlan::from_allowed(union_allowed.as_ref());
    let tsconfig = resolve_tsconfig(root, tsconfig_path)?;
    let (required_graph_plan, _) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan(root, plan);
    if required_graph_plan.is_empty() && shared.graph_files().is_empty() {
        return super::check(root, config, tsconfig_path);
    }
    if shared.stats.parse_errors > 0 {
        return super::check(root, config, tsconfig_path);
    }
    if !shared.graph_plan().covers(required_graph_plan) {
        bail!(
            "shared check facts are missing graph facts required by {RULE_ID}; collect facts with forbidden_dependencies::graph_plan before calling run_check_with_facts"
        );
    }
    let graph = DepGraph::build_with_plan_file_list_and_check_facts(
        root,
        &tsconfig,
        plan,
        shared.graph_files().to_vec(),
        shared,
    );
    let mut findings = Vec::new();
    for (rule, opts) in applications.iter().zip(opts_list.iter()) {
        findings.extend(check_rule_application(root, config, rule, opts, &graph)?);
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}
