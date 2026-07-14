use super::{
    rust_max_lines_per_file, rust_no_inline_allows, rust_no_inline_tests, RuleFinding,
    RUST_MAX_LINES_PER_FILE, RUST_NO_INLINE_ALLOWS, RUST_NO_INLINE_TESTS,
};
use crate::codebase::ts_source::has_disable_file_comment;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod scan;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct MaxLinesOptions {
    src_max: Option<usize>,
    test_max: Option<usize>,
    excludes: Vec<String>,
    roots: Option<Vec<PathBuf>>,
}

#[derive(Default)]
pub(super) struct RustWork {
    pub(super) max_limits: Vec<usize>,
    pub(super) inline_tests: bool,
    pub(super) inline_allows: bool,
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    exclusive_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let mut work = BTreeMap::<PathBuf, RustWork>::new();
    add_max_lines_work(root, config, all_files, &mut work)?;
    add_inline_tests_work(root, config, all_files, &mut work)?;
    add_inline_allows_work(root, config, all_files, &mut work)?;

    let mut findings: Vec<RuleFinding> = work
        .par_iter()
        .flat_map(|(path, work)| {
            scan::scan_file(
                root,
                path,
                work,
                exclusive_files.binary_search(path).is_ok(),
                sources,
            )
        })
        .collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn add_max_lines_work(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    work: &mut BTreeMap<PathBuf, RustWork>,
) -> Result<()> {
    for rule in config.rule_applications(RUST_MAX_LINES_PER_FILE) {
        let opts: MaxLinesOptions = rule.rule_options();
        let files =
            matching_rust_files(root, config, rule, all_files, &opts.excludes, &opts.roots)?;
        for path in files {
            let limit = if rust_max_lines_per_file::is_test_file(root, &path) {
                opts.test_max
                    .unwrap_or(rust_max_lines_per_file::DEFAULT_TEST_MAX)
            } else {
                opts.src_max
                    .unwrap_or(rust_max_lines_per_file::DEFAULT_SRC_MAX)
            };
            let entry = work.entry(path).or_default();
            if !entry.max_limits.contains(&limit) {
                entry.max_limits.push(limit);
            }
        }
    }
    Ok(())
}

fn add_inline_tests_work(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    work: &mut BTreeMap<PathBuf, RustWork>,
) -> Result<()> {
    for rule in config.rule_applications(RUST_NO_INLINE_TESTS) {
        let opts: MaxLinesOptions = rule.rule_options();
        let files =
            matching_rust_files(root, config, rule, all_files, &opts.excludes, &opts.roots)?;
        for path in files
            .into_iter()
            .filter(|path| !rust_max_lines_per_file::is_test_file(root, path))
        {
            work.entry(path).or_default().inline_tests = true;
        }
    }
    Ok(())
}

fn add_inline_allows_work(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    work: &mut BTreeMap<PathBuf, RustWork>,
) -> Result<()> {
    for rule in config.rule_applications(RUST_NO_INLINE_ALLOWS) {
        let opts: MaxLinesOptions = rule.rule_options();
        let files =
            matching_rust_files(root, config, rule, all_files, &opts.excludes, &opts.roots)?;
        for path in files
            .into_iter()
            .filter(|path| !rust_max_lines_per_file::is_test_file(root, path))
        {
            work.entry(path).or_default().inline_allows = true;
        }
    }
    Ok(())
}

fn matching_rust_files(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
    all_files: &[PathBuf],
    excludes: &[String],
    option_roots: &Option<Vec<PathBuf>>,
) -> Result<Vec<PathBuf>> {
    let target_roots = super::target_roots(root, config, rule);
    let roots = normalize_roots(root, &target_roots, option_roots);
    let skip = super::skip_dir_set(config);
    let files: Vec<PathBuf> = all_files
        .iter()
        .filter(|path| {
            super::file_allowed_by_roots_and_skip(root, &skip, path, &roots)
                && path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension == "rs")
                && !is_excluded(root, path, excludes)
        })
        .cloned()
        .collect();
    super::path_filter::filter_rule_files(root, config, rule, &files)
}

fn normalize_roots(
    root: &Path,
    target_roots: &[PathBuf],
    option_roots: &Option<Vec<PathBuf>>,
) -> Vec<PathBuf> {
    option_roots
        .as_deref()
        .map(|roots| {
            roots
                .iter()
                .map(|path| {
                    if path.is_absolute() {
                        path.clone()
                    } else {
                        root.join(path)
                    }
                })
                .collect()
        })
        .unwrap_or_else(|| target_roots.to_vec())
}

fn is_excluded(root: &Path, path: &Path, excludes: &[String]) -> bool {
    let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
    excludes
        .iter()
        .any(|exclude| rel.contains(exclude.as_str()))
}

#[cfg(test)]
mod tests;
