use super::RuleFinding;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, NodeId};
use crate::codebase::dependencies::{relationship_filter, EdgeKind, RelationshipArg};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use helpers::{build_globset, edge_kind_str, repro_command, resolve_root_path};
use std::collections::HashSet;
use std::path::Path;

mod config_check;
mod helpers;
mod shared;
pub(crate) use config_check::check_with_config;
pub(crate) use shared::{
    check_with_prepared_facts_and_graph, check_with_prepared_facts_and_session,
    check_with_prepared_facts_inferred_and_session, PreparedForbiddenCheckRequest,
};

pub const RULE_ID: &str = "forbidden-dependencies";

#[derive(Debug, serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub roots: Vec<String>,
    pub forbidden_modules: Vec<String>,
    pub forbidden_files: Vec<String>,
    pub relationships: Vec<RelationshipArg>,
}

/// Resolves the `DepGraph`'s `GraphConfigOptions` via default `--config`
/// discovery. Callers that already have an explicit `config_path` in scope
/// must use [`check_with_config`] instead, or a custom config changing
/// route/http/queue/dotnet/swift/terraform graph settings is silently
/// ignored for this rule's graph build.
pub fn check(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig_path: Option<&Path>,
) -> Result<Vec<RuleFinding>> {
    check_with_config(root, config, None, tsconfig_path)
}

pub fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    let applications = config.rule_applications(RULE_ID);
    if applications.is_empty() {
        return None;
    }
    let opts_list: Vec<Options> = applications.iter().map(|r| r.rule_options()).collect();
    let union_allowed = union_allowed_set(&opts_list);
    Some(GraphBuildPlan::from_allowed(union_allowed.as_ref()))
}

fn union_allowed_set(opts_list: &[Options]) -> Option<HashSet<EdgeKind>> {
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

fn check_rule_application(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
    opts: &Options,
    graph: &DepGraph,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
) -> Result<Vec<RuleFinding>> {
    if opts.roots.is_empty()
        || (opts.forbidden_modules.is_empty() && opts.forbidden_files.is_empty())
    {
        return Ok(vec![RuleFinding {
            rule: RULE_ID.to_string(),
            file: ".no-mistakes.yml".to_string(),
            line: 1,
            message: format!(
                "{RULE_ID}: each rule entry requires at least one root and \
                 one forbiddenModules or forbiddenFiles entry"
            ),
            import: None,
            target: None,
        }]);
    }
    let module_matcher = match build_globset(&opts.forbidden_modules) {
        Ok(m) => m,
        Err(e) => {
            return Ok(vec![RuleFinding {
                rule: RULE_ID.to_string(),
                file: ".no-mistakes.yml".to_string(),
                line: 1,
                message: format!("{RULE_ID}: invalid glob pattern in forbiddenModules: {e}"),
                import: None,
                target: None,
            }]);
        }
    };
    let file_matcher = match build_globset(&opts.forbidden_files) {
        Ok(m) => m,
        Err(e) => {
            return Ok(vec![RuleFinding {
                rule: RULE_ID.to_string(),
                file: ".no-mistakes.yml".to_string(),
                line: 1,
                message: format!("{RULE_ID}: invalid glob pattern in forbiddenFiles: {e}"),
                import: None,
                target: None,
            }]);
        }
    };
    let allowed = relationship_filter(&opts.relationships);
    let mut inferred_roots = inferred_roots.cloned().unwrap_or_default();
    let source_filter = super::path_filter::RulePathFilter::new_with_inferred(
        root,
        config,
        rule,
        &mut inferred_roots,
    )?;
    let mut findings = Vec::new();
    for root_str in &opts.roots {
        let Some(resolved_path) = resolve_root_path(root, root_str) else {
            continue;
        };
        if !source_filter.is_match(&resolved_path) {
            continue;
        }
        let file = resolved_path
            .strip_prefix(root)
            .unwrap_or(&resolved_path)
            .to_string_lossy()
            .replace('\\', "/");
        let root_node = NodeId::File(resolved_path);
        let entries = graph.deps_of(&[root_node], None, allowed.as_ref());
        for entry in &entries {
            let matched = match &entry.node {
                NodeId::Module(spec) => module_matcher.as_ref().is_some_and(|m| m.is_match(spec)),
                NodeId::File(path) | NodeId::Symbol { file: path, .. } => {
                    file_matcher.as_ref().is_some_and(|m| {
                        let rel = path.strip_prefix(root).unwrap_or(path);
                        m.is_match(rel.to_string_lossy().replace('\\', "/"))
                    })
                }
                NodeId::QueueJob { .. } => false,
            };
            if !matched {
                continue;
            }
            let target_name = entry.node.display_name(root).replace('\\', "/");
            let via: Vec<String> = entry.via.iter().map(edge_kind_str).collect();
            let kind = match &entry.node {
                NodeId::Module(_) => "module",
                _ => "file",
            };
            let repro = repro_command(root_str, &target_name, &entry.node, &opts.relationships);
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: file.clone(),
                line: 1,
                message: format!(
                    "{root_str} reaches forbidden {kind} '{target_name}' via {}. \
                     Reproduce: {repro}",
                    via.join(","),
                ),
                import: Some(via.join(",")),
                target: Some(target_name),
            });
        }
    }
    Ok(findings)
}

#[cfg(test)]
mod tests;
