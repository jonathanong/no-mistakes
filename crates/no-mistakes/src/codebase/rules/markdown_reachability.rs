//! Enforces the deliberately small documentation discovery graph used by agents.
use super::RuleFinding;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod baseline;
mod finding;
mod graph;
mod scope_state;
mod state;

use baseline::{read_baseline, BaselineEntry};
use finding::{finding, stale};
use scope_state::{scoped_states, ScopeOptions};
use state::{expected_state, RuleState};

pub const RULE_ID: &str = "markdown-reachability";
const DEFAULT_ROOT_FILENAMES: &[&str] = &["CLAUDE.md"];
const DEFAULT_INDEX_FILENAMES: &[&str] = &["README.md"];
const DEFAULT_MAX_DEPTH: usize = 2;
type RuleStates = BTreeMap<String, RuleState>;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    root_filenames: Option<Vec<String>>,
    index_filenames: Option<Vec<String>>,
    max_depth: Option<usize>,
    baseline_file: Option<PathBuf>,
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    facts: &super::markdown_facts::MarkdownFactMap,
) -> Result<Vec<RuleFinding>> {
    let markdown = super::markdown_scope::markdown_files(all_files);
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let options: Options = rule.try_rule_options()?;
        let roots = filenames(&options.root_filenames, DEFAULT_ROOT_FILENAMES);
        let indexes = filenames(&options.index_filenames, DEFAULT_INDEX_FILENAMES);
        let max_depth = validate_max_depth(options.max_depth)?;
        let target_paths =
            super::path_filter::filter_markdown_rule_files(root, config, rule, &markdown)?;
        let scope_roots = super::markdown_scope::scope_roots(root, config, rule);
        let scope_options = ScopeOptions {
            roots: &roots,
            indexes: &indexes,
            max_depth,
            facts,
        };
        let (states, target_names) =
            scoped_states(root, &scope_roots, &markdown, &target_paths, scope_options)?;
        let baseline = read_baseline(root, options.baseline_file.as_deref(), all_files)?;
        collect_findings(
            &mut findings,
            root,
            states,
            &target_names,
            &baseline,
            &scope_roots,
            max_depth,
        )?;
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn collect_findings(
    findings: &mut Vec<RuleFinding>,
    root: &Path,
    states: RuleStates,
    target_names: &BTreeSet<String>,
    baseline: &BTreeMap<String, BaselineEntry>,
    scope_roots: &[PathBuf],
    max_depth: usize,
) -> Result<()> {
    for (baseline_key, state) in states {
        let expected = expected_state(state.depth, state.allowed);
        match (expected, baseline.get(&baseline_key)) {
            (None, Some(_)) => findings.push(stale(
                &state.finding_file,
                "is reachable; remove its baseline entry",
            )),
            (None, None) => {}
            (Some(expected), Some(actual)) if actual == &expected => {}
            (Some(expected), Some(_)) => findings.push(stale(
                &state.finding_file,
                &format!("baseline does not match current {}", expected.state),
            )),
            (Some(expected), None) => findings.push(finding(
                &state.finding_file,
                &expected,
                max_depth,
                state.invalid_intermediary,
            )),
        }
    }
    for file in baseline.keys() {
        if !target_names.contains(file) {
            let finding_file =
                super::markdown_scope::baseline_finding_key(root, scope_roots, file, RULE_ID)?;
            findings.push(stale(
                &finding_file,
                "references a deleted or excluded Markdown file",
            ));
        }
    }
    Ok(())
}

fn validate_max_depth(configured: Option<usize>) -> Result<usize> {
    let depth = configured.unwrap_or(DEFAULT_MAX_DEPTH);
    if !(1..=2).contains(&depth) {
        anyhow::bail!("{RULE_ID} options.maxDepth must be 1 or 2; README-only discovery supports no deeper graph")
    }
    Ok(depth)
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
