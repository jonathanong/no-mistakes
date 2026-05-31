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

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct MaxLinesOptions {
    src_max: Option<usize>,
    test_max: Option<usize>,
    excludes: Vec<String>,
    roots: Option<Vec<PathBuf>>,
}

#[derive(Default)]
struct RustWork {
    max_limits: Vec<usize>,
    inline_tests: bool,
    inline_allows: bool,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let mut work = BTreeMap::<PathBuf, RustWork>::new();
    add_max_lines_work(root, config, all_files, &mut work)?;
    add_inline_tests_work(root, config, all_files, &mut work)?;
    add_inline_allows_work(root, config, all_files, &mut work)?;

    let mut findings: Vec<RuleFinding> = work
        .par_iter()
        .flat_map(|(path, work)| scan_file(root, path, work))
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

fn scan_file(root: &Path, path: &Path, work: &RustWork) -> Vec<RuleFinding> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };

    let mut findings = Vec::new();
    for limit in &work.max_limits {
        if let Some(finding) = rust_max_lines_per_file::check_source(path, root, &content, *limit) {
            findings.push(finding);
        }
    }

    let inline_tests_enabled =
        work.inline_tests && !has_disable_file_comment(&content, RUST_NO_INLINE_TESTS);
    let inline_allows_enabled =
        work.inline_allows && !has_disable_file_comment(&content, RUST_NO_INLINE_ALLOWS);
    let needs_inline_tests_parse =
        inline_tests_enabled && content.contains("cfg") && content.contains("test");
    let needs_inline_allows_parse = inline_allows_enabled && content.contains("allow");
    if needs_inline_tests_parse || needs_inline_allows_parse {
        if let Ok(parsed) = syn::parse_file(&content) {
            if needs_inline_tests_parse {
                findings.extend(rust_no_inline_tests::findings_from_parsed(
                    path, root, &parsed,
                ));
            }
            if needs_inline_allows_parse {
                findings.extend(rust_no_inline_allows::findings_from_parsed(
                    path, root, &parsed,
                ));
            }
        }
    }

    dedup_findings(findings)
}

fn dedup_findings(mut findings: Vec<RuleFinding>) -> Vec<RuleFinding> {
    findings.sort();
    findings.dedup();
    findings
}

#[cfg(test)]
mod tests;
