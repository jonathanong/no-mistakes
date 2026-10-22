use super::RuleFinding;
use crate::codebase::dependencies::graph::{DepGraph, NodeId};
use crate::codebase::dependencies::{relationship_filter, RelationshipArg};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use helpers::{build_globset, edge_kind_str, repro_command, resolve_root_paths};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

mod config_check;
mod helpers;
#[cfg(not(test))]
macro_rules! record_inventory_scan {
    ($root:expr) => {};
}
#[cfg(test)]
#[macro_use]
mod inventory_scan_tests;
mod plan;
mod shared;
pub(crate) use config_check::check_with_config;
pub(crate) use plan::try_graph_plan_for_files;
pub(crate) use plan::try_graph_plan_for_files_inventory_only;
pub use plan::{graph_plan, try_graph_plan};
pub(crate) use shared::check_with_prepared_facts_and_graph_declared;
pub(crate) use shared::SharedGraphScope;

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

fn check_rule_application(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
    opts: &Options,
    graph: &DepGraph,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
    file_universe: Option<&crate::fx::PathSet>,
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
    );
    let source_filter = source_filter?;
    let mut findings = Vec::new();
    let mut selected_roots = BTreeMap::new();
    for root_str in &opts.roots {
        for path in resolve_root_paths(root, root_str, graph) {
            selected_roots.entry(path).or_insert(root_str);
        }
    }
    for (resolved_path, root_str) in selected_roots {
        if !source_filter.is_match(&resolved_path) {
            continue;
        }
        let file = resolved_path
            .strip_prefix(root)
            .unwrap_or(&resolved_path)
            .to_string_lossy()
            .replace('\\', "/");
        let root_node = NodeId::file(resolved_path);
        let entries = match file_universe {
            Some(universe) => {
                graph.deps_of_in_file_universe(&[root_node], None, allowed.as_ref(), universe)
            }
            None => graph.deps_of(&[root_node], None, allowed.as_ref()),
        };
        for entry in &entries {
            let matched = match &entry.node {
                NodeId::Module(spec) => module_matcher
                    .as_ref()
                    .is_some_and(|m| m.is_match(spec.as_ref())),
                NodeId::File(path) | NodeId::Symbol { file: path, .. } => {
                    file_matcher.as_ref().is_some_and(|m| {
                        let rel = path.strip_prefix(root).unwrap_or(path);
                        m.is_match(rel.to_string_lossy().replace('\\', "/"))
                    })
                }
                NodeId::QueueJob { .. }
                | NodeId::WorkflowJob { .. }
                | NodeId::WorkflowStep { .. }
                | NodeId::TrpcProcedure { .. } => false,
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
            let repro = repro_command(&file, &target_name, &entry.node, &opts.relationships);
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
mod inventory_tests;
#[cfg(test)]
mod public_prepared_tests;
#[cfg(test)]
mod tests;
