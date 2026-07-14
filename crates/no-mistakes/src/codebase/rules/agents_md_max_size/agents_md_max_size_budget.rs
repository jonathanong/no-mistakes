use super::{Options, DEFAULT_MAX_CHARS, DEFAULT_MAX_LINES, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path, SourceStore};
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub(super) fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let sources = crate::codebase::rules::source_store_for_files(files);
    scan_with_sources(root, opts, files, &sources)
}

pub(super) fn scan_with_sources(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    let max_lines = opts.max_lines.unwrap_or(DEFAULT_MAX_LINES);
    let max_chars = opts.max_chars.unwrap_or(DEFAULT_MAX_CHARS);
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| {
            let Some(content) = crate::codebase::rules::read_source(sources, path) else {
                return Vec::new();
            };
            check_content(path, root, max_lines, max_chars, &content)
        })
        .collect();
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

pub(super) fn scan_advisories_with_sources(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    let max_chars = opts.max_chars.unwrap_or(DEFAULT_MAX_CHARS);
    let threshold = opts.advisory_chars_remaining.unwrap_or_default();
    let mut advisories: Vec<RuleFinding> = files
        .par_iter()
        .filter_map(|path| {
            let content = crate::codebase::rules::read_source(sources, path)?;
            check_advisory_content(path, root, max_chars, threshold, &content)
        })
        .collect();
    advisories.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(advisories)
}

pub(super) fn check_content(
    path: &Path,
    root: &Path,
    max_lines: usize,
    max_chars: usize,
    content: &str,
) -> Vec<RuleFinding> {
    if has_disable_file_comment(content, RULE_ID) {
        return Vec::new();
    }
    let file = relative_slash_path(root, path);
    let mut findings = Vec::new();
    let line_count = count_lines(content);
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
                "{} - trim to keep agent context lean",
                format_char_budget(content, char_count, max_chars)
            ),
            import: None,
            target: None,
        });
    }
    findings
}

fn check_advisory_content(
    path: &Path,
    root: &Path,
    max_chars: usize,
    threshold: usize,
    content: &str,
) -> Option<RuleFinding> {
    if has_disable_file_comment(content, RULE_ID) {
        return None;
    }
    let char_count = content.chars().count();
    if char_count > max_chars {
        return None;
    }
    let remaining = max_chars - char_count;
    if remaining > threshold {
        return None;
    }
    Some(RuleFinding {
        rule: RULE_ID.to_string(),
        file: relative_slash_path(root, path),
        line: 1,
        message: format!(
            "{} - consider moving detail into linked docs before editing",
            format_char_budget(content, char_count, max_chars)
        ),
        import: None,
        target: None,
    })
}

fn format_char_budget(content: &str, char_count: usize, max_chars: usize) -> String {
    let byte_count = content.len();
    if char_count > max_chars {
        let over = char_count - max_chars;
        format!("{char_count} characters / {byte_count} bytes (max {max_chars}, {over} over)")
    } else {
        let remaining = max_chars - char_count;
        format!(
            "{char_count} characters / {byte_count} bytes (max {max_chars}, {remaining} remaining)"
        )
    }
}

pub(super) fn count_lines(content: &str) -> usize {
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
