use super::RuleFinding;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, NodeId};
use crate::codebase::dependencies::{relationship_filter, EdgeKind};
use crate::codebase::rules::config_diagnostic::ConfigAnchor;
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use helpers::{edge_kind_str, repro_command};
use std::collections::HashSet;
use std::path::Path;

mod config_check;
mod helpers;
mod shared;
pub(crate) use config_check::check_with_config;
pub(crate) use shared::{check_with_prepared_facts_and_graph, PreparedConfig};

pub const RULE_ID: &str = "forbidden-dependencies";

mod options;
mod selectors;
use options::Options;

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

/// Builds the graph plan with the historical default fallback for malformed
/// options. Check entrypoints should use [`try_graph_plan`] instead.
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

/// Fallible graph-plan construction for check, CLI, and N-API request paths.
pub fn try_graph_plan(config: &NoMistakesConfig) -> Result<Option<GraphBuildPlan>> {
    let applications = config.rule_applications(RULE_ID);
    if applications.is_empty() {
        return Ok(None);
    }
    let opts_list: Vec<Options> = applications
        .iter()
        .map(|rule| rule.try_rule_options())
        .collect::<Result<_>>()?;
    let union_allowed = union_allowed_set(&opts_list);
    Ok(Some(GraphBuildPlan::from_allowed(union_allowed.as_ref())))
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

struct ApplicationScope<'a> {
    inferred_roots: Option<&'a crate::codebase::config::InferredRoots>,
    file_universe: Option<&'a crate::fx::PathSet>,
    sources: Option<&'a SourceStore>,
}

fn application_scope<'a>(
    inferred_roots: Option<&'a crate::codebase::config::InferredRoots>,
    file_universe: Option<&'a crate::fx::PathSet>,
    sources: Option<&'a SourceStore>,
) -> ApplicationScope<'a> {
    ApplicationScope {
        inferred_roots,
        file_universe,
        sources,
    }
}

fn check_rule_application(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
    opts: &Options,
    graph: &DepGraph,
    scope: ApplicationScope<'_>,
) -> Result<Vec<RuleFinding>> {
    let anchor = ConfigAnchor::resolve(root, config, scope.sources, rule);
    let (module_matcher, file_matcher) = match helpers::config_matchers(&anchor, opts) {
        Ok(matchers) => matchers,
        Err(findings) => return Ok(findings),
    };
    let selectors = match selectors::Selectors::new(root, &anchor, opts) {
        Ok(selectors) => selectors,
        Err(finding) => return Ok(vec![finding]),
    };
    let allowed = relationship_filter(&opts.relationships);
    let mut inferred_roots = scope.inferred_roots.cloned().unwrap_or_default();
    let source_filter = super::path_filter::RulePathFilter::new_with_inferred(
        root,
        config,
        rule,
        &mut inferred_roots,
    );
    let source_filter = source_filter?;
    let mut findings = Vec::new();
    for (resolved_path, root_str) in selectors.select(graph, scope.file_universe) {
        if !source_filter.is_match(&resolved_path) {
            continue;
        }
        let file = resolved_path
            .strip_prefix(root)
            .unwrap_or(&resolved_path)
            .to_string_lossy()
            .replace('\\', "/");
        let root_node = NodeId::file(resolved_path.clone());
        let entries = match scope.file_universe {
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
                            && !selectors.allows(&resolved_path, path)
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
mod tests;
