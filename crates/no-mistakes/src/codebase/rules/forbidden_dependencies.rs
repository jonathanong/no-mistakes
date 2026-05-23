use super::RuleFinding;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, NodeId};
use crate::codebase::dependencies::{relationship_filter, EdgeKind, RelationshipArg};
use crate::codebase::ts_resolver::{find_tsconfig as ts_find, load_tsconfig, TsConfig};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::HashSet;
use std::path::Path;

pub const RULE_ID: &str = "forbidden-dependencies";

#[derive(Debug, serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub roots: Vec<String>,
    pub forbidden_modules: Vec<String>,
    pub forbidden_files: Vec<String>,
    pub relationships: Vec<RelationshipArg>,
}

pub fn check(
    root: &Path,
    config: &NoMistakesConfig,
    tsconfig_path: Option<&Path>,
) -> Result<Vec<RuleFinding>> {
    let applications = config.rule_applications(RULE_ID);
    if applications.is_empty() {
        return Ok(Vec::new());
    }
    let opts_list: Vec<Options> = applications.iter().map(|r| r.rule_options()).collect();
    let union_allowed = union_allowed_set(&opts_list);
    let plan = GraphBuildPlan::from_allowed(union_allowed.as_ref());
    let tsconfig = resolve_tsconfig(root, tsconfig_path)?;
    let graph = DepGraph::build_with_plan(root, &tsconfig, plan)?;
    let mut findings = Vec::new();
    for opts in &opts_list {
        findings.extend(check_application(root, opts, &graph)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn resolve_tsconfig(root: &Path, tsconfig_path: Option<&Path>) -> Result<TsConfig> {
    match tsconfig_path {
        Some(path) => load_tsconfig(path),
        None => match ts_find(root) {
            Some(path) => load_tsconfig(&path),
            None => Ok(TsConfig {
                dir: root.to_path_buf(),
                paths: vec![],
                paths_dir: root.to_path_buf(),
                base_url: None,
            }),
        },
    }
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

fn check_application(root: &Path, opts: &Options, graph: &DepGraph) -> Result<Vec<RuleFinding>> {
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
    let module_matcher = build_globset(&opts.forbidden_modules)?;
    let file_matcher = build_globset(&opts.forbidden_files)?;
    let allowed = relationship_filter(&opts.relationships);
    let mut findings = Vec::new();
    for root_str in &opts.roots {
        let Some(root_node) = resolve_root_node(root, root_str) else {
            continue;
        };
        let entries = graph.deps_of(&[root_node], None, allowed.as_ref());
        for entry in &entries {
            let matched = match &entry.node {
                NodeId::Module(spec) => module_matcher.as_ref().is_some_and(|m| m.is_match(spec)),
                NodeId::File(path) => file_matcher.as_ref().is_some_and(|m| {
                    let rel = path.strip_prefix(root).unwrap_or(path);
                    m.is_match(rel.to_string_lossy().replace('\\', "/"))
                }),
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
            let file = std::path::Path::new(root_str)
                .strip_prefix(root)
                .map(|rel| rel.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| root_str.replace('\\', "/"));
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file,
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

fn resolve_root_node(root: &Path, raw: &str) -> Option<NodeId> {
    let p = std::path::Path::new(raw);
    let path = if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(raw)
    };
    let normalized = crate::codebase::ts_resolver::normalize_path(&path);
    normalized.exists().then_some(NodeId::File(normalized))
}

fn build_globset(patterns: &[String]) -> Result<Option<GlobSet>> {
    if patterns.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for p in patterns {
        builder.add(Glob::new(p)?);
    }
    Ok(Some(builder.build()?))
}

fn edge_kind_str(k: &EdgeKind) -> String {
    serde_json::to_value(k)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}

fn repro_command(
    root_str: &str,
    target_name: &str,
    node: &NodeId,
    relationships: &[RelationshipArg],
) -> String {
    let target_flag = match node {
        NodeId::Module(_) => format!("--target-module '{}'", target_name.replace('\'', "'\\''")),
        _ => format!("--filter '{}'", target_name.replace('\'', "'\\''")),
    };
    let rel_flags = if relationships.is_empty() {
        " --relationship all".to_string()
    } else {
        relationships
            .iter()
            .map(|r| {
                format!(
                    " --relationship {}",
                    serde_json::to_value(r).unwrap().as_str().unwrap()
                )
            })
            .collect::<String>()
    };
    format!(
        "no-mistakes dependencies '{}' {target_flag}{rel_flags} --format json",
        root_str.replace('\'', "'\\''")
    )
}

#[cfg(test)]
mod tests;
