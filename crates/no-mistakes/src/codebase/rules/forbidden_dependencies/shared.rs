use super::{check_rule_application, resolve_tsconfig, union_allowed_set, Options, RULE_ID};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::rules::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
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
    let graph = DepGraph::build_with_plan_file_list_and_check_facts(
        root,
        &tsconfig,
        plan,
        shared.files().to_vec(),
        shared,
    );
    let mut findings = Vec::new();
    for (rule, opts) in applications.iter().zip(opts_list.iter()) {
        findings.extend(check_rule_application(root, config, rule, opts, &graph)?);
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}
