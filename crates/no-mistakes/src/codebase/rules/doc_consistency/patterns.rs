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

/// An empty match is never reported. A branch that matches "" also hides a
/// later alternative at that position, because `find_iter` is leftmost-first.
fn empty_match_hint(pattern: &str) -> Option<&'static str> {
    let hir = regex_syntax::parse(pattern).ok()?;
    if hir.properties().minimum_len() != Some(0) {
        return None;
    }
    if alternation_hides_later_branch(&hir) {
        Some(
            "so that branch hides later alternatives; use `+` instead of `*` or drop the empty branch",
        )
    } else {
        Some(
            "and empty matches are never reported; require at least one character (e.g. `+` instead of `*`, drop a trailing `?`, anchor with a literal)",
        )
    }
}

fn alternation_hides_later_branch(hir: &regex_syntax::hir::Hir) -> bool {
    let kind = hir.kind();
    let hides = matches!(
        kind,
        regex_syntax::hir::HirKind::Alternation(branches)
            if branches
                .iter()
                .any(|branch| branch.properties().minimum_len() == Some(0))
    );
    hides || kind.subs().iter().any(alternation_hides_later_branch)
}

fn compile_pattern(index: usize, pattern: &str) -> Result<Regex> {
    if let Some(hint) = empty_match_hint(pattern) {
        bail!(
            "doc-consistency bannedPatterns[{index}].pattern `{pattern}` can match the empty string, {hint}"
        );
    }
    RegexBuilder::new(pattern)
        .size_limit(BANNED_PATTERN_SIZE_LIMIT)
        .build()
        .map_err(|error| pattern_regex_error(index, pattern, error))
}

fn pattern_regex_error(index: usize, pattern: &str, error: regex::Error) -> anyhow::Error {
    if matches!(error, regex::Error::CompiledTooBig(_)) {
        anyhow::anyhow!(
            "doc-consistency bannedPatterns[{index}].pattern `{pattern}` exceeds the 10 MiB compiled-size limit; reduce repetition counts"
        )
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
