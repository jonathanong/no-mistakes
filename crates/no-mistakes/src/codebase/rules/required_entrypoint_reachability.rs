use super::RuleFinding;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, GraphBuildPlan, NodeId};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "required-entrypoint-reachability";

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) source_globs: Vec<String>,
    pub(crate) entrypoints: Vec<String>,
    pub(crate) max_depth: Option<usize>,
}

pub(crate) fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    config
        .rule_configured(RULE_ID)
        .then(|| GraphBuildPlan::from_allowed(Some(&runtime_edge_kinds())))
}

pub(crate) fn check_with_graph_and_inferred(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    graph: &DepGraph,
    inferred_roots: Option<&crate::codebase::config::InferredRoots>,
) -> Result<Vec<RuleFinding>> {
    let file_universe = files
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<crate::fx::PathSet>();
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.try_rule_options()?;
        let mut inferred_roots = inferred_roots.cloned().unwrap_or_default();
        let source_filter = super::path_filter::RulePathFilter::new_with_inferred(
            root,
            config,
            rule,
            &mut inferred_roots,
        );
        let source_filter = source_filter?;
        let scoped_files = files
            .iter()
            .filter(|path| source_filter.is_match(path))
            .cloned()
            .collect::<Vec<_>>();
        let target_roots =
            super::target_roots_with_inferred(root, config, rule, &mut inferred_roots);
        findings.extend(check_rule_application(
            root,
            &options,
            &scoped_files,
            &target_roots,
            graph,
            &file_universe,
        ));
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn check_rule_application(
    root: &Path,
    options: &Options,
    scoped_files: &[PathBuf],
    target_roots: &[PathBuf],
    graph: &DepGraph,
    file_universe: &crate::fx::PathSet,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    if options.source_globs.is_empty() {
        findings.push(config_finding(
            "each rule entry requires at least one sourceGlobs pattern",
            None,
        ));
    }
    if options.entrypoints.is_empty() {
        findings.push(config_finding(
            "each rule entry requires at least one entrypoint",
            None,
        ));
    }

    let mut source_files = HashSet::new();
    for pattern in &options.source_globs {
        match super::matching_files(
            root,
            std::slice::from_ref(pattern),
            scoped_files,
            target_roots,
        ) {
            Ok(matches) if matches.is_empty() => findings.push(config_finding(
                &format!("sourceGlobs pattern `{pattern}` matched no files"),
                Some(pattern.clone()),
            )),
            Ok(matches) => source_files.extend(matches),
            Err(error) => findings.push(config_finding(
                &format!("invalid sourceGlobs pattern `{pattern}`: {error}"),
                Some(pattern.clone()),
            )),
        }
    }

    let mut entrypoint_paths = Vec::new();
    let mut entrypoint_labels = Vec::new();
    for configured in &options.entrypoints {
        let path = resolve_entrypoint(root, configured);
        if path
            .as_ref()
            .is_none_or(|path| !file_universe.contains(path) || !graph.contains_file(path))
        {
            findings.push(config_finding(
                &format!("entrypoint `{configured}` does not exist"),
                Some(configured.clone()),
            ));
            continue;
        }
        let path = path.expect("validated entrypoint path");
        entrypoint_labels.push(relative_slash_path(root, &path));
        entrypoint_paths.push(path);
    }
    entrypoint_labels.sort();
    entrypoint_labels.dedup();
    entrypoint_paths.sort();
    entrypoint_paths.dedup();

    if !entrypoint_paths.is_empty() {
        let allowed = runtime_edge_kinds();
        let roots = entrypoint_paths
            .iter()
            .cloned()
            .map(NodeId::file)
            .collect::<Vec<_>>();
        let mut reachable = graph
            .deps_of_in_file_universe(&roots, options.max_depth, Some(&allowed), file_universe)
            .into_iter()
            .filter_map(|entry| entry.node.as_file().map(Path::to_path_buf))
            .collect::<HashSet<_>>();
        reachable.extend(entrypoint_paths);
        let target = entrypoint_labels.join(",");
        for source in source_files {
            let source = crate::codebase::ts_resolver::normalize_path(&source);
            if reachable.contains(&source) {
                continue;
            }
            let file = relative_slash_path(root, &source);
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: file.clone(),
                line: 1,
                message: format!(
                    "{file} is not runtime-reachable from configured entrypoints: {target}"
                ),
                import: None,
                target: Some(target.clone()),
            });
        }
    }
    findings
}

fn resolve_entrypoint(root: &Path, configured: &str) -> Option<PathBuf> {
    let configured = Path::new(configured.trim_start_matches("./"));
    let path = if configured.is_absolute() {
        configured.to_path_buf()
    } else {
        root.join(configured)
    };
    let path = crate::codebase::ts_resolver::normalize_path(&path);
    path.starts_with(root).then_some(path)
}

fn runtime_edge_kinds() -> HashSet<EdgeKind> {
    [
        EdgeKind::Import,
        EdgeKind::DynamicImport,
        EdgeKind::Require,
        EdgeKind::WorkspaceImport,
        EdgeKind::AssetImport,
    ]
    .into_iter()
    .collect()
}

fn config_finding(message: &str, target: Option<String>) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: ".no-mistakes.yml".to_string(),
        line: 1,
        message: format!("{RULE_ID}: {message}"),
        import: None,
        target,
    }
}

#[cfg(test)]
mod tests;
