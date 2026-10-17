use super::super::RuleFinding;
use super::RULE_ID;
use super::options::BannedPattern;
use crate::codebase::ts_source::byte_offset_to_line;
use anyhow::{Result, bail};
use regex::Regex;

pub(super) struct CompiledPattern {
    regex: Regex,
    globs: super::super::path_filter::GlobMatcher,
    message: String,
}

/// Compile every pattern once. An empty list does not build a regex or a glob.
pub(super) fn compile(patterns: &[BannedPattern]) -> Result<Vec<CompiledPattern>> {
    if patterns.is_empty() {
        return Ok(Vec::new());
    }
    patterns
        .iter()
        .enumerate()
        .map(|(index, pattern)| compile_one(index, pattern))
        .collect()
}

fn compile_one(index: usize, pattern: &BannedPattern) -> Result<CompiledPattern> {
    let globs: Vec<String> = pattern
        .files
        .iter()
        .map(|glob| glob.trim().to_string())
        .collect();
    if globs.is_empty() {
        bail!("doc-consistency bannedPatterns[{index}].files must list at least one glob");
    }
    if globs.iter().any(|glob| glob.is_empty()) {
        bail!("doc-consistency bannedPatterns[{index}].files must not contain an empty glob");
    }
    if pattern.pattern.is_empty() {
        bail!("doc-consistency bannedPatterns[{index}].pattern must not be empty");
    }
    let message = pattern.message.trim();
    if message.is_empty() {
        bail!("doc-consistency bannedPatterns[{index}].message must not be empty");
    }
    let regex = Regex::new(&pattern.pattern).map_err(|error| {
        anyhow::anyhow!(
            "doc-consistency bannedPatterns[{index}].pattern is an invalid regex: {error}"
        )
    })?;
    let globs = super::super::path_filter::GlobMatcher::new(
        &globs,
        &format!("doc-consistency bannedPatterns[{index}].files"),
    )?;
    Ok(CompiledPattern {
        regex,
        globs,
        message: message.to_string(),
    })
}

pub(super) fn findings_in_file(
    rel: &str,
    content: &str,
    compiled: &[CompiledPattern],
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for pattern in compiled {
        if !pattern.globs.is_match(rel) {
            continue;
        }
        for line in match_start_lines(&pattern.regex, content) {
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.to_string(),
                line,
                message: pattern.message.clone(),
                import: None,
                target: None,
            });
        }
    }
    findings
}

// `find_iter` is leftmost-first and non-overlapping. Empty matches are not
// findings; the iterator still advances one Unicode scalar after each empty
// match, so a leading empty alternative can hide a later alternative there.
fn match_start_lines(regex: &Regex, text: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    for found in regex.find_iter(text) {
        if found.start() == found.end() {
            continue;
        }
        let line = byte_offset_to_line(text, found.start()) as usize;
        if lines.last().copied() != Some(line) {
            lines.push(line);
        }
    }
    lines
}
