use super::super::path_filter::GlobMatcher;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub(crate) const DEFAULT_SRC_MAX: usize = 200;
pub(crate) const DEFAULT_TEST_MAX: usize = 500;
pub(super) const DEFAULT_GENERATED_EXCLUDE: &str = "**/*.g.cs";
pub(super) const DEFAULT_TEST_ROOTS: &[&str] = &["**/tests/**", "**/*.Tests/**"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) src_max: Option<usize>,
    pub(crate) test_max: Option<usize>,
    pub(crate) excludes: Vec<String>,
    pub(crate) roots: Option<Vec<PathBuf>>,
    pub(crate) test_roots: Option<Vec<String>>,
}

pub(super) struct PreparedRule {
    src_max: usize,
    test_max: usize,
    pub(super) roots: Vec<PathBuf>,
    excludes: Vec<String>,
    exclude_globs: GlobMatcher,
    pub(super) test_roots: GlobMatcher,
}

impl PreparedRule {
    pub(super) fn new(opts: &Options, root: &Path, target_roots: &[PathBuf]) -> Result<Self> {
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

    pub(super) fn is_excluded(&self, root: &Path, path: &Path) -> bool {
        let rel = relative_slash_path(root, path);
        rel.ends_with(".g.cs")
            || self
                .excludes
                .iter()
                .any(|exclude| !exclude.is_empty() && rel.contains(exclude.as_str()))
            || self.exclude_globs.is_match(&rel)
    }

    pub(super) fn limit_for(&self, root: &Path, path: &Path) -> usize {
        if is_test_file(root, path, &self.test_roots) {
            self.test_max
        } else {
            self.src_max
        }
    }
}

pub(super) fn is_test_file(root: &Path, path: &Path, test_roots: &GlobMatcher) -> bool {
    let rel = relative_slash_path(root, path);
    rel.contains("/tests/") || rel.starts_with("tests/") || test_roots.is_match(&rel)
}

pub(super) fn is_csharp_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("cs")
}

pub(super) fn count_physical_lines(content: &str) -> usize {
    if content.is_empty() {
        0
    } else {
        content.split('\n').count() - if content.ends_with('\n') { 1 } else { 0 }
    }
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
