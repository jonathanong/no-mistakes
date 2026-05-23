use super::RuleFinding;
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
    let rel = file.as_str();
    let ext = rel.rfind('.').map(|i| &rel[i..]).unwrap_or("");

    if content.trim().is_empty() {
        return vec![RuleFinding {
            rule: RULE_ID.to_string(),
            file,
            line: 1,
            message: "file is empty".to_string(),
            import: None,
            target: None,
        }];
    }

    if !has_non_comment_content(&content, ext) {
        return vec![RuleFinding {
            rule: RULE_ID.to_string(),
            file,
            line: 1,
            message: "file contains only comments — add real content or remove it".to_string(),
            import: None,
            target: None,
        }];
    }

    Vec::new()
}

pub(crate) fn has_non_comment_content(content: &str, ext: &str) -> bool {
    let stripped = match ext {
        ".ts" | ".mts" | ".cts" | ".tsx" | ".js" | ".jsx" | ".mjs" | ".cjs" | ".rs" | ".css" => {
            strip_comments(content, "//", "/*", "*/")
        }
        ".sql" => strip_comments(content, "--", "/*", "*/"),
        ".md" => strip_html_comments(content),
        _ => content.to_string(),
    };
    stripped.split_whitespace().next().is_some()
}

/// Strip line comments (prefix) and block comments (open/close) from content.
fn strip_comments(content: &str, line_prefix: &str, block_open: &str, block_close: &str) -> String {
    let mut result = String::new();
    let mut s = content;
    while !s.is_empty() {
        if s.starts_with(block_open) {
            if let Some(close) = s.find(block_close) {
                s = &s[close + block_close.len()..];
            } else {
                break;
            }
        } else if s.starts_with(line_prefix) {
            s = s.find('\n').map(|i| &s[i..]).unwrap_or("");
        } else {
            let mut chars = s.chars();
            if let Some(c) = chars.next() {
                result.push(c);
                s = chars.as_str();
            }
        }
    }
    result
}

/// Strip `<!-- ... -->` blocks (may span lines).
fn strip_html_comments(content: &str) -> String {
    let mut result = String::new();
    let mut s = content;
    while !s.is_empty() {
        if let Some(open) = s.find("<!--") {
            result.push_str(&s[..open]);
            s = s[open..]
                .find("-->")
                .map(|i| &s[open + i + 3..])
                .unwrap_or("");
        } else {
            result.push_str(s);
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests;
