use super::RuleFinding;
use crate::codebase::ts_source::{
    byte_offset_to_line, discover_with_extensions, has_disable_file_comment, relative_slash_path,
};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub const RULE_ID: &str = "rust-no-inline-tests";

static INLINE_TEST_RE: OnceLock<Regex> = OnceLock::new();

fn inline_test_re() -> &'static Regex {
    INLINE_TEST_RE.get_or_init(|| {
        Regex::new(
            r"(?s)#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*(?:(?:#\s*\[[^\]]*\]|//[^\n]*|/\*.*?\*/)\s*)*(?:pub(?:\([^)]*\))?\s+)?mod\s+(?:r#)?\w+\s*\{"
        )
        .expect("inline test regex is valid")
    })
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) excludes: Vec<String>,
    pub(crate) roots: Option<Vec<PathBuf>>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let skip = &config.filesystem.skip_directories;
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = rule.rule_options();
        let target_roots = super::target_roots(root, config, rule);
        let roots = normalize_roots(&opts, root, &target_roots);
        let files: Vec<PathBuf> = roots
            .iter()
            .flat_map(|r| discover_with_extensions(r, skip, &["rs"]))
            .filter(|p| {
                !is_excluded(root, p, &opts.excludes)
                    && !super::rust_max_lines_per_file::is_test_file(root, p)
            })
            .collect();
        findings.extend(scan(root, &opts, &files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

/// Check using a pre-discovered file list to avoid a second filesystem walk.
pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = rule.rule_options();
        let target_roots = super::target_roots(root, config, rule);
        let roots = normalize_roots(&opts, root, &target_roots);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|p| {
                roots.iter().any(|r| p.starts_with(r))
                    && p.extension()
                        .and_then(|e| e.to_str())
                        .is_some_and(|e| e == "rs")
                    && !is_excluded(root, p, &opts.excludes)
                    && !super::rust_max_lines_per_file::is_test_file(root, p)
            })
            .cloned()
            .collect();
        findings.extend(scan(root, &opts, &files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn normalize_roots(opts: &Options, root: &Path, target_roots: &[PathBuf]) -> Vec<PathBuf> {
    opts.roots
        .as_deref()
        .map(|rs| {
            rs.iter()
                .map(|r| {
                    if r.is_absolute() {
                        r.clone()
                    } else {
                        root.join(r)
                    }
                })
                .collect()
        })
        .unwrap_or_else(|| target_roots.to_vec())
}

fn is_excluded(root: &Path, path: &Path, excludes: &[String]) -> bool {
    let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
    excludes.iter().any(|e| rel.contains(e.as_str()))
}

fn scan(root: &Path, _opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file(path, root))
        .collect();
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    Ok(findings)
}

pub(crate) fn check_file(path: &Path, root: &Path) -> Vec<RuleFinding> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    if has_disable_file_comment(&content, RULE_ID) {
        return Vec::new();
    }
    let file = relative_slash_path(root, path);
    inline_test_re()
        .find_iter(&content)
        .map(|m| RuleFinding {
            rule: RULE_ID.to_string(),
            file: file.clone(),
            line: byte_offset_to_line(&content, m.start()) as usize,
            message: "inline #[cfg(test)] mod block - use #[cfg(test)] mod tests; with a sibling tests.rs".to_string(),
            import: None,
            target: None,
        })
        .collect()
}

#[cfg(test)]
mod tests;
