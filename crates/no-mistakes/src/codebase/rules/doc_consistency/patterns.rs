use super::super::RuleFinding;
use super::options::BannedPattern;
use super::RULE_ID;
use crate::codebase::ts_source::byte_offset_to_line;
use anyhow::{bail, Result};
use regex::{Regex, RegexBuilder};

pub(super) struct CompiledPattern {
    regex: Regex,
    globs: super::super::path_filter::GlobMatcher,
    message: String,
}

/// Same 10 MiB ceiling as `Regex::new`. Set explicitly so an oversized pattern
/// is `CompiledTooBig` instead of a generic invalid-regex error.
const BANNED_PATTERN_SIZE_LIMIT: usize = 10 * (1 << 20);

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
    let regex = compile_pattern(index, &pattern.pattern)?;
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

/// `find_iter` is leftmost-first, so an alternative that matches "" hides a
/// later alternative at that position. Reject that at config load.
fn pattern_can_match_empty(pattern: &str) -> bool {
    regex_syntax::parse(pattern).is_ok_and(|hir| hir.properties().minimum_len() == Some(0))
}

fn compile_pattern(index: usize, pattern: &str) -> Result<Regex> {
    if pattern_can_match_empty(pattern) {
        bail!("doc-consistency bannedPatterns[{index}].pattern can match the empty string");
    }
    RegexBuilder::new(pattern)
        .size_limit(BANNED_PATTERN_SIZE_LIMIT)
        .build()
        .map_err(|error| pattern_regex_error(index, error))
}

fn pattern_regex_error(index: usize, error: regex::Error) -> anyhow::Error {
    if matches!(error, regex::Error::CompiledTooBig(_)) {
        anyhow::anyhow!("doc-consistency bannedPatterns[{index}].pattern is a pattern too large")
    } else {
        anyhow::anyhow!(
            "doc-consistency bannedPatterns[{index}].pattern is an invalid regex: {error}"
        )
    }
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

/// Patterns that can match "" are rejected at compile time. This skip remains
/// so a zero-width engine match is still not reported as a finding.
pub(super) fn match_start_lines(regex: &Regex, text: &str) -> Vec<usize> {
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
