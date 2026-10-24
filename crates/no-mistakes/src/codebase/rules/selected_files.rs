use super::RuleFinding;
use crate::codebase::ts_source::discover_files;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use anyhow::Result;
use rayon::prelude::*;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

/// Select an application's files from the request-owned visible-file inventory.
pub(crate) fn selected_rule_files(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    all_files: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    selected_rule_files_with_roots(root, config, rule, all_files).map(|(files, _)| files)
}

pub(crate) fn selected_rule_files_with_roots(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    all_files: &[PathBuf],
) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let (files, target_roots) = rule_scope_files_with_roots(root, config, rule, all_files);
    let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
    Ok((files, target_roots))
}

/// Preserve companion-file visibility before an application's include/exclude filter.
pub(crate) fn rule_scope_files_with_roots(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    all_files: &[PathBuf],
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let target_roots = super::target_roots(root, config, rule);
    let skip = super::skip_dir_set(config);
    let files: Vec<PathBuf> = all_files
        .iter()
        .filter(|path| super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots))
        .cloned()
        .collect();
    (files, target_roots)
}

pub(crate) fn selected_rule_files_matching(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    all_files: &[PathBuf],
    matches: impl Fn(&Path) -> bool,
) -> Result<Vec<PathBuf>> {
    let target_roots = super::target_roots(root, config, rule);
    let skip = super::skip_dir_set(config);
    let files: Vec<PathBuf> = all_files
        .iter()
        .filter(|path| {
            matches(path) && super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots)
        })
        .cloned()
        .collect();
    super::path_filter::filter_rule_files(root, config, rule, &files)
}

/// Standalone rule entry points discover each configured target before filtering.
pub(crate) fn discovered_rule_files(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
) -> Result<Vec<PathBuf>> {
    let target_roots = super::target_roots(root, config, rule);
    let files: Vec<PathBuf> = target_roots
        .iter()
        .flat_map(|target| discover_files(target, &config.filesystem.skip_directories))
        .collect();
    super::path_filter::filter_rule_files(root, config, rule, &files)
}

/// Run a filesystem-only rule over either standalone discovery or a prepared inventory.
pub(crate) fn run_filesystem_rule<O: DeserializeOwned + Default + Send>(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: Option<&[PathBuf]>,
    rule_id: &str,
    scan: impl Fn(&Path, &O, &[PathBuf]) -> Result<Vec<RuleFinding>> + Send + Sync,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(rule_id)
        .into_par_iter()
        .map(|rule| {
            let options: O = rule.try_rule_options()?;
            let files = match all_files {
                Some(all_files) => selected_rule_files(root, config, rule, all_files)?,
                None => discovered_rule_files(root, config, rule)?,
            };
            scan(root, &options, &files)
        })
        .collect();
    let mut findings = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

pub(crate) fn check_indexed_standalone(
    root: &Path,
    config: &NoMistakesConfig,
    check: impl FnOnce(&Path, &NoMistakesConfig, &[PathBuf]) -> Result<Vec<RuleFinding>>,
) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let files = discover_files(&root, &config.filesystem.skip_directories);
    let files: Vec<_> = files
        .into_iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
        .collect();
    check(&root, config, &files)
}

pub(crate) fn run_selected_workflow_rule<O: DeserializeOwned + Default, C>(
    context: super::SourcedRuleContext<'_>,
    rule_id: &str,
    compile: impl Fn(O) -> C,
    include: impl Fn(&C) -> &[String],
    scan: impl Fn(&Path, &C, &[PathBuf], &crate::codebase::ts_source::SourceStore) -> Vec<RuleFinding>,
) -> Result<Vec<RuleFinding>> {
    let super::SourcedRuleContext {
        root,
        config,
        all_files,
        sources,
    } = context;
    super::run_rule_applications(config, rule_id, |rule| {
        let compiled = compile(rule.try_rule_options()?);
        let (files, target_roots) = selected_rule_files_with_roots(root, config, rule, all_files)?;
        if files.is_empty() {
            return Ok(Vec::new());
        }
        let files = super::matching_files(root, include(&compiled), &files, &target_roots)?;
        Ok(scan(root, &compiled, &files, sources))
    })
}

pub(crate) fn run_sourced_file_rule<O: DeserializeOwned + Default>(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    rule_id: &str,
    scan: impl Fn(
        &Path,
        &O,
        &[PathBuf],
        &crate::codebase::ts_source::SourceStore,
    ) -> Result<Vec<RuleFinding>>,
) -> Result<Vec<RuleFinding>> {
    super::run_rule_applications(config, rule_id, |rule| {
        let options: O = rule.try_rule_options()?;
        let files = selected_rule_files(root, config, rule, all_files)?;
        scan(root, &options, &files, sources)
    })
}

pub(crate) fn configured_includes(include: Vec<String>, defaults: &[&str]) -> Vec<String> {
    if include.is_empty() {
        defaults
            .iter()
            .map(|pattern| (*pattern).to_string())
            .collect()
    } else {
        include
    }
}

pub(crate) fn check_sourced_standalone(
    root: &Path,
    config: &NoMistakesConfig,
    check: impl FnOnce(&Path, &NoMistakesConfig, &[PathBuf]) -> Result<Vec<RuleFinding>>,
) -> Result<Vec<RuleFinding>> {
    let files = discover_files(root, &config.filesystem.skip_directories);
    check(root, config, &files)
}

pub(crate) struct RuleGlobFilter {
    include: super::path_filter::GlobMatcher,
    exclude: super::path_filter::GlobMatcher,
}

impl RuleGlobFilter {
    pub(crate) fn new(rule_id: &str, include: &[String], exclude: &[String]) -> Result<Self> {
        Ok(Self {
            include: super::path_filter::GlobMatcher::new(include, &format!("{rule_id} include"))?,
            exclude: super::path_filter::GlobMatcher::new(exclude, &format!("{rule_id} exclude"))?,
        })
    }

    pub(crate) fn includes(&self, rel: &str) -> bool {
        (self.include.is_empty() || self.include.is_match(rel))
            && (self.exclude.is_empty() || !self.exclude.is_match(rel))
    }
}
