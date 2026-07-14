use super::{check_rule_application, union_allowed_set, Options, RULE_ID};
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, GraphFiles};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_resolver::TsConfig;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

/// Same as [`super::check`], but resolves the `DepGraph`'s
/// `GraphConfigOptions` from an explicit `--config` path instead of always
/// falling back to default discovery.
pub(crate) fn check_with_config(
    root: &Path,
    config: &NoMistakesConfig,
    config_path: Option<&Path>,
    tsconfig_path: Option<&Path>,
) -> Result<Vec<RuleFinding>> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let root = root.as_path();
    let graph_files = GraphFiles::discover(root);
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
        tsconfig_path,
        root,
        graph_files.all(),
    )?;
    check_with_config_tsconfig_and_files(root, config, config_path, &tsconfig, &graph_files)
}

fn check_with_config_tsconfig_and_files(
    root: &Path,
    config: &NoMistakesConfig,
    config_path: Option<&Path>,
    tsconfig: &TsConfig,
    graph_files: &GraphFiles,
) -> Result<Vec<RuleFinding>> {
    let applications = config.rule_applications(RULE_ID);
    if applications.is_empty() {
        return Ok(Vec::new());
    }
    let opts_list: Vec<Options> = applications
        .iter()
        .map(|rule| rule.rule_options())
        .collect();
    let union_allowed = union_allowed_set(&opts_list);
    let plan = GraphBuildPlan::from_allowed(union_allowed.as_ref());
    let graph =
        DepGraph::build_with_plan_and_files_config(root, tsconfig, plan, graph_files, config_path)?;
    let mut findings = Vec::new();
    for (rule, opts) in applications.iter().zip(opts_list.iter()) {
        findings.extend(check_rule_application(
            root, config, rule, opts, &graph, None,
        )?);
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}
