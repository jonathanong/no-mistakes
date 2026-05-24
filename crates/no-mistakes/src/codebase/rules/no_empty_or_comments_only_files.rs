use super::RuleFinding;
use crate::codebase::comment_only::{classify_content, ContentKind};
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "no-empty-or-comments-only-files";

const DEFAULT_EXTENSIONS: &[&str] = &[
    ".ts", ".mts", ".cts", ".tsx", ".js", ".jsx", ".sql", ".rs", ".css",
];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) extensions: Vec<String>,
    pub(crate) intentionally_empty: Vec<String>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let skip = &config.filesystem.skip_directories;
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        let target_roots = super::target_roots(root, config, rule);
        let files: Vec<PathBuf> = target_roots
            .iter()
            .flat_map(|r| discover_files(r, skip))
            .collect();
        findings.extend(scan(root, &opts, &files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        let target_roots = super::target_roots(root, config, rule);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|p| target_roots.iter().any(|r| p.starts_with(r)))
            .cloned()
            .collect();
        findings.extend(scan(root, &opts, &files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn effective_extensions(opts: &Options) -> Vec<&str> {
    if opts.extensions.is_empty() {
        DEFAULT_EXTENSIONS.to_vec()
    } else {
        opts.extensions.iter().map(String::as_str).collect()
    }
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let extensions = effective_extensions(opts);
    let exempt: HashSet<&str> = opts
        .intentionally_empty
        .iter()
        .map(String::as_str)
        .collect();
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| {
            let rel = relative_slash_path(root, path);
            if exempt.contains(rel.as_str()) {
                return Vec::new();
            }
            let has_ext = extensions.iter().any(|ext| rel.ends_with(*ext));
            if !has_ext {
                return Vec::new();
            }
            check_file(path, root)
        })
        .collect();
    findings.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(findings)
}

pub(crate) fn check_file(path: &Path, root: &Path) -> Vec<RuleFinding> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let file = relative_slash_path(root, path);
    let ext_no_dot = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let message = match classify_content(&content, ext_no_dot) {
        ContentKind::HasContent => return Vec::new(),
        ContentKind::Empty => "file is empty",
        ContentKind::CommentsOnly => "file contains only comments — add real content or remove it",
    };
    vec![RuleFinding {
        rule: RULE_ID.to_string(),
        file,
        line: 1,
        message: message.to_string(),
        import: None,
        target: None,
    }]
}

#[cfg(test)]
mod tests;
