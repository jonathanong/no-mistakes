use super::prepared::PreparedRule;
use super::RuleFinding;
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path, SourceStore};
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub(super) fn scan(
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
                    let content = crate::codebase::rules::read_source(sources, path)?;
                    check_source(path, root, &content, limit, defer_suppression)
                }
                None => check_file(path, root, limit, defer_suppression),
            }
        })
        .collect();
    findings.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(findings)
}

pub(super) fn check_file(
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

pub(super) fn check_source(
    path: &Path,
    root: &Path,
    content: &str,
    limit: usize,
    defer_suppression: bool,
) -> Option<RuleFinding> {
    if !defer_suppression && has_disable_file_comment(content, super::RULE_ID) {
        return None;
    }
    let physical_lines = super::prepared::count_physical_lines(content);
    if physical_lines <= limit {
        return None;
    }
    Some(RuleFinding {
        rule: super::RULE_ID.to_string(),
        file: relative_slash_path(root, path),
        line: 1,
        message: format!(
            "{physical_lines} physical lines (max {limit}) - split into smaller types or files"
        ),
        import: None,
        target: None,
    })
}
