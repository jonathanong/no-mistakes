use super::RuleFinding;
use crate::codebase::ts_source::{
    discover_with_basenames, has_disable_file_comment, relative_slash_path,
};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "agents-md-max-size";

const DEFAULT_MAX_LINES: usize = 200;
const DEFAULT_MAX_CHARS: usize = 12_000;
const DEFAULT_FILENAMES: &[&str] = &["AGENTS.md", "CLAUDE.md"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) max_lines: Option<usize>,
    pub(crate) max_chars: Option<usize>,
    pub(crate) filenames: Option<Vec<String>>,
    pub(crate) roots: Option<Vec<PathBuf>>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let skip = &config.filesystem.skip_directories;
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts = rule.rule_options();
        let filenames = filenames_from_opts(&opts);
        let target_roots = super::target_roots(root, config, rule);
        let roots = roots_from_opts(&opts, root, &target_roots);
        let files: Vec<PathBuf> = roots
            .iter()
            .flat_map(|r| discover_with_basenames(r, skip, &filenames))
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
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
        let filenames = filenames_from_opts(&opts);
        let target_roots = super::target_roots(root, config, rule);
        let roots = roots_from_opts(&opts, root, &target_roots);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|p| {
                super::file_allowed_by_roots_and_skip(root, &skip, p, &roots)
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| filenames.contains(&n))
            })
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan(root, &opts, &files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn filenames_from_opts(opts: &Options) -> Vec<&str> {
    opts.filenames
        .as_deref()
        .map(|v| v.iter().map(String::as_str).collect())
        .unwrap_or_else(|| DEFAULT_FILENAMES.to_vec())
}

fn roots_from_opts(opts: &Options, root: &Path, target_roots: &[PathBuf]) -> Vec<PathBuf> {
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

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let max_lines = opts.max_lines.unwrap_or(DEFAULT_MAX_LINES);
    let max_chars = opts.max_chars.unwrap_or(DEFAULT_MAX_CHARS);
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file(path, root, max_lines, max_chars))
        .collect();
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

fn check_file(path: &Path, root: &Path, max_lines: usize, max_chars: usize) -> Vec<RuleFinding> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    if has_disable_file_comment(&content, RULE_ID) {
        return Vec::new();
    }
    let file = relative_slash_path(root, path);
    let mut findings = Vec::new();
    let line_count = count_lines(&content);
    if line_count > max_lines {
        findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: file.clone(),
            line: 1,
            message: format!(
                "{line_count} lines (max {max_lines}) - trim to keep agent context lean"
            ),
            import: None,
            target: None,
        });
    }
    let char_count = content.chars().count();
    if char_count > max_chars {
        findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file,
            line: 1,
            message: format!(
                "{char_count} characters (max {max_chars}) - trim to keep agent context lean"
            ),
            import: None,
            target: None,
        });
    }
    findings
}

pub(crate) fn count_lines(content: &str) -> usize {
    if content.is_empty() {
        return 0;
    }
    let newlines = content.bytes().filter(|&b| b == b'\n').count();
    if content.ends_with('\n') {
        newlines
    } else {
        newlines + 1
    }
}

#[cfg(test)]
mod tests;
