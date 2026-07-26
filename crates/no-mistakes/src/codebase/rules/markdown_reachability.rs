//! Enforces the deliberately small documentation discovery graph used by agents.
use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod baseline;
mod finding;
mod graph;

use baseline::{read_baseline, BaselineEntry};
use finding::{finding, stale};
use graph::{direct_or_readme_hop, link_graph, shortest_depth};

pub const RULE_ID: &str = "markdown-reachability";
const DEFAULT_ROOT_FILENAMES: &[&str] = &["CLAUDE.md"];
const DEFAULT_INDEX_FILENAMES: &[&str] = &["README.md"];
const DEFAULT_MAX_DEPTH: usize = 2;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    root_filenames: Option<Vec<String>>,
    index_filenames: Option<Vec<String>>,
    max_depth: Option<usize>,
    baseline_file: Option<PathBuf>,
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let markdown = markdown_files(root, all_files);
    let graph = link_graph(root, &markdown, sources);
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.rule_options();
        let roots = filenames(&options.root_filenames, DEFAULT_ROOT_FILENAMES);
        let indexes = filenames(&options.index_filenames, DEFAULT_INDEX_FILENAMES);
        let max_depth = validate_max_depth(options.max_depth)?;
        let target_paths = super::path_filter::filter_rule_files(root, config, rule, &markdown)?;
        let target_names = target_paths
            .iter()
            .filter(|path| !is_named(path, &roots))
            .map(|path| relative_slash_path(root, path))
            .collect::<BTreeSet<_>>();
        let states = target_paths
            .iter()
            .filter(|path| !is_named(path, &roots))
            .map(|path| {
                let depth = shortest_depth(path, &roots, &graph);
                let allowed = direct_or_readme_hop(path, &roots, &indexes, &graph, max_depth);
                (relative_slash_path(root, path), (depth, allowed))
            })
            .collect::<BTreeMap<_, _>>();
        let baseline = read_baseline(root, options.baseline_file.as_deref(), all_files)?;
        collect_findings(&mut findings, states, &target_names, &baseline, max_depth);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn collect_findings(
    findings: &mut Vec<RuleFinding>,
    states: BTreeMap<String, (Option<usize>, bool)>,
    target_names: &BTreeSet<String>,
    baseline: &BTreeMap<String, BaselineEntry>,
    max_depth: usize,
) {
    for (file, (depth, allowed)) in states {
        let expected = expected_state(depth, allowed);
        match (expected, baseline.get(&file)) {
            (None, Some(_)) => {
                findings.push(stale(&file, "is reachable; remove its baseline entry"))
            }
            (None, None) => {}
            (Some(expected), Some(actual)) if actual == &expected => {}
            (Some(expected), Some(_)) => findings.push(stale(
                &file,
                &format!("baseline does not match current {}", expected.state),
            )),
            (Some(expected), None) => findings.push(finding(&file, &expected, max_depth)),
        }
    }
    for file in baseline.keys() {
        if !target_names.contains(file) {
            findings.push(stale(
                file,
                "references a deleted or excluded Markdown file",
            ));
        }
    }
}

fn expected_state(depth: Option<usize>, allowed: bool) -> Option<BaselineEntry> {
    (!allowed).then(|| match depth {
        Some(depth) => BaselineEntry::depth(depth),
        None => BaselineEntry::unreachable(),
    })
}

fn validate_max_depth(configured: Option<usize>) -> Result<usize> {
    let depth = configured.unwrap_or(DEFAULT_MAX_DEPTH);
    if !(1..=2).contains(&depth) {
        anyhow::bail!("{RULE_ID} options.maxDepth must be 1 or 2; README-only discovery supports no deeper graph")
    }
    Ok(depth)
}

fn markdown_files(root: &Path, files: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = files
        .iter()
        .filter(|path| path.starts_with(root) && path.extension().is_some_and(|ext| ext == "md"))
        .cloned()
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    files
}

fn filenames(configured: &Option<Vec<String>>, defaults: &[&str]) -> BTreeSet<String> {
    configured
        .as_ref()
        .map(|items| items.iter().cloned().collect())
        .unwrap_or_else(|| defaults.iter().map(|item| (*item).to_string()).collect())
}

fn is_named(path: &Path, names: &BTreeSet<String>) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| names.contains(name))
}

#[cfg(test)]
#[path = "markdown_reachability/tests.rs"]
mod tests;
