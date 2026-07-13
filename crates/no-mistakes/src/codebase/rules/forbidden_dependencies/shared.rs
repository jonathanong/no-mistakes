use super::{check_rule_application, union_allowed_set, Options, RULE_ID};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use std::path::Path;

pub(crate) fn check_with_prepared_facts(
    root: &Path,
    config: &NoMistakesConfig,
    config_path: Option<&Path>,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    prepared_graph: Option<&crate::codebase::dependencies::graph::PreparedGraphConfig>,
) -> Result<Vec<RuleFinding>> {
    check_with_optional_inferred(
        root,
        config,
        config_path,
        tsconfig,
        shared,
        prepared_graph,
        None,
    )
}

pub(crate) fn check_with_prepared_facts_and_inferred(
    root: &Path,
    config: &NoMistakesConfig,
    config_path: Option<&Path>,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    prepared_graph: Option<&crate::codebase::dependencies::graph::PreparedGraphConfig>,
    inferred_roots: &crate::codebase::config::InferredRoots,
) -> Result<Vec<RuleFinding>> {
    check_with_optional_inferred(
        root,
        config,
        config_path,
        tsconfig,
        shared,
        prepared_graph,
        Some(inferred_roots),
    )
}

fn check_with_optional_inferred(
    root: &Path,
    config: &NoMistakesConfig,
    config_path: Option<&Path>,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    shared: &crate::codebase::check_facts::CheckFactMap,
    prepared_graph: Option<&crate::codebase::dependencies::graph::PreparedGraphConfig>,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
) -> Result<Vec<RuleFinding>> {
    let applications = config.rule_applications(RULE_ID);
    if applications.is_empty() {
        return Ok(Vec::new());
    }
    let opts_list: Vec<Options> = applications.iter().map(|r| r.rule_options()).collect();
    let union_allowed = union_allowed_set(&opts_list);
    let plan = GraphBuildPlan::from_allowed(union_allowed.as_ref());
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
    let graph = match prepared_graph {
        Some(prepared) => DepGraph::build_with_plan_file_list_prepared_config_and_check_facts(
            root,
            tsconfig,
            plan,
            shared.graph_file_universe().to_vec(),
            config_path,
            shared,
            prepared,
        )?,
        None => DepGraph::build_with_plan_file_list_config_and_complete_check_facts(
            root,
            tsconfig,
            plan,
            shared.graph_file_universe().to_vec(),
            config_path,
            shared,
        )?,
    };
    let mut findings = Vec::new();
    for (rule, opts) in applications.iter().zip(opts_list.iter()) {
        findings.extend(check_rule_application(
            root,
            config,
            rule,
            opts,
            &graph,
            inferred_roots,
        )?);
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}
