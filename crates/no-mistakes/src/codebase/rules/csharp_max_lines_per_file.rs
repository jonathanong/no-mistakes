use super::path_filter::GlobMatcher;
use super::RuleFinding;
use crate::codebase::ts_source::{
    discover_with_extensions, has_disable_file_comment, relative_slash_path, SourceStore,
};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "csharp-max-lines-per-file";

pub(crate) const DEFAULT_SRC_MAX: usize = 200;
pub(crate) const DEFAULT_TEST_MAX: usize = 500;
const DEFAULT_GENERATED_EXCLUDE: &str = "**/*.g.cs";
const DEFAULT_TEST_ROOTS: &[&str] = &["**/tests/**", "**/*.Tests/**"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) src_max: Option<usize>,
    pub(crate) test_max: Option<usize>,
    pub(crate) excludes: Vec<String>,
    pub(crate) roots: Option<Vec<PathBuf>>,
    pub(crate) test_roots: Option<Vec<String>>,
}

struct PreparedRule {
    src_max: usize,
    test_max: usize,
    roots: Vec<PathBuf>,
    excludes: Vec<String>,
    exclude_globs: GlobMatcher,
    test_roots: GlobMatcher,
}

impl PreparedRule {
    fn new(opts: &Options, root: &Path, target_roots: &[PathBuf]) -> Result<Self> {
        let excludes = effective_excludes(opts);
        let test_root_patterns = effective_test_roots(opts);
        Ok(Self {
            src_max: opts.src_max.unwrap_or(DEFAULT_SRC_MAX),
            test_max: opts.test_max.unwrap_or(DEFAULT_TEST_MAX),
            roots: normalize_roots(opts, root, target_roots),
            exclude_globs: GlobMatcher::new(&excludes, "csharp-max-lines-per-file excludes")?,
            excludes,
            test_roots: GlobMatcher::new(
                &test_root_patterns,
                "csharp-max-lines-per-file testRoots",
            )?,
        })
    }

    fn is_excluded(&self, root: &Path, path: &Path) -> bool {
        let rel = relative_slash_path(root, path);
        rel.ends_with(".g.cs")
            || self
                .excludes
                .iter()
                .any(|exclude| !exclude.is_empty() && rel.contains(exclude.as_str()))
            || self.exclude_globs.is_match(&rel)
    }

    fn limit_for(&self, root: &Path, path: &Path) -> usize {
        if is_test_file(root, path, &self.test_roots) {
            self.test_max
        } else {
            self.src_max
        }
    }
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let skip = &config.filesystem.skip_directories;
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = rule.rule_options();
        let prepared = PreparedRule::new(&opts, root, &super::target_roots(root, config, rule))?;
        let files: Vec<PathBuf> = prepared
            .roots
            .iter()
            .flat_map(|rule_root| discover_with_extensions(rule_root, skip, &["cs"]))
            .filter(|path| !prepared.is_excluded(root, path))
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan(root, &prepared, &files, None, false)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    check_with_files_sources_and_deferred_suppression(root, config, all_files, sources, false)
}

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let skip = super::skip_dir_set(config);
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = rule.rule_options();
        let prepared = PreparedRule::new(&opts, root, &super::target_roots(root, config, rule))?;
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|path| {
                is_csharp_file(path)
                    && super::file_allowed_by_roots_and_skip(root, &skip, path, &prepared.roots)
                    && !prepared.is_excluded(root, path)
            })
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan(
            root,
            &prepared,
            &files,
            Some(sources),
            defer_suppression,
        )?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(
    root: &Path,
    prepared: &PreparedRule,
    files: &[PathBuf],
    sources: Option<&SourceStore>,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .filter_map(|path| {
            let limit = prepared.limit_for(root, path);
            match sources {
                Some(sources) => {
                    let content = super::read_source(sources, path)?;
                    check_source(path, root, &content, limit, defer_suppression)
                }
                None => check_file(path, root, limit, defer_suppression),
            }
        })
        .collect();
    findings.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(findings)
}

fn check_file(
    path: &Path,
    root: &Path,
    limit: usize,
    defer_suppression: bool,
) -> Option<RuleFinding> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return None;
    };
    check_source(path, root, &content, limit, defer_suppression)
}

fn check_source(
    path: &Path,
    root: &Path,
    content: &str,
    limit: usize,
    defer_suppression: bool,
) -> Option<RuleFinding> {
    if !defer_suppression && has_disable_file_comment(content, RULE_ID) {
        return None;
    }
    let physical_lines = count_physical_lines(content);
    if physical_lines <= limit {
        return None;
    }
    Some(RuleFinding {
        rule: RULE_ID.to_string(),
        file: relative_slash_path(root, path),
        line: 1,
        message: format!(
            "{physical_lines} physical lines (max {limit}) - split into smaller types or files"
        ),
        import: None,
        target: None,
    })
}

fn count_physical_lines(content: &str) -> usize {
    if content.is_empty() {
        0
    } else {
        content.split('\n').count() - if content.ends_with('\n') { 1 } else { 0 }
    }
}

fn is_test_file(root: &Path, path: &Path, test_roots: &GlobMatcher) -> bool {
    let rel = relative_slash_path(root, path);
    rel.contains("/tests/") || rel.starts_with("tests/") || test_roots.is_match(&rel)
}

fn is_csharp_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("cs")
}

fn normalize_roots(opts: &Options, root: &Path, target_roots: &[PathBuf]) -> Vec<PathBuf> {
    opts.roots
        .as_deref()
        .map(|roots| {
            roots
                .iter()
                .map(|rule_root| {
                    if rule_root.is_absolute() {
                        rule_root.clone()
                    } else {
                        root.join(rule_root)
                    }
                })
                .collect()
        })
        .unwrap_or_else(|| target_roots.to_vec())
}

fn effective_excludes(opts: &Options) -> Vec<String> {
    let mut excludes = opts.excludes.clone();
    if !excludes
        .iter()
        .any(|exclude| exclude == DEFAULT_GENERATED_EXCLUDE)
    {
        excludes.push(DEFAULT_GENERATED_EXCLUDE.to_string());
    }
    excludes
}

fn effective_test_roots(opts: &Options) -> Vec<String> {
    match &opts.test_roots {
        Some(roots) => roots.clone(),
        None => DEFAULT_TEST_ROOTS
            .iter()
            .map(|pattern| (*pattern).to_string())
            .collect(),
    }
}

#[cfg(test)]
mod tests;
