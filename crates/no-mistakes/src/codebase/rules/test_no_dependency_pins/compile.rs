use super::{CompiledPattern, RULE_ID};
use anyhow::{Context, Result};
use regex::Regex;

const LOOKBEHIND_NOT_AT: &str = "(?<!@)";

/// Compiles a user-supplied pattern: every match is reported as written.
pub(super) fn compile_pattern(
    reason: &str,
    source: &str,
    multiline: bool,
) -> Result<CompiledPattern> {
    let (pattern, reject_preceding_at) = match source.strip_prefix(LOOKBEHIND_NOT_AT) {
        Some(rest) => (rest, true),
        None => (source, false),
    };
    let regex = Regex::new(pattern)
        .with_context(|| format!("{RULE_ID} contains invalid pattern `{source}`"))?;
    Ok(CompiledPattern {
        reason: reason.to_string(),
        regex,
        reject_preceding_at,
        multiline,
        builtin: false,
        line_context: None,
    })
}

/// Compiles a default pattern, which reports its `pin` capture, skips
/// placeholder values, and only runs on lines matching `line_context`.
pub(super) fn compile_builtin(
    reason: &str,
    source: &str,
    multiline: bool,
    line_context: Option<&str>,
) -> Result<CompiledPattern> {
    let mut pattern = compile_pattern(reason, source, multiline)?;
    pattern.builtin = true;
    pattern.line_context = line_context
        .map(|context| {
            Regex::new(context)
                .with_context(|| format!("{RULE_ID} contains invalid line context `{context}`"))
        })
        .transpose()?;
    Ok(pattern)
}

#[cfg(test)]
mod tests;
