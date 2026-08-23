use super::super::RuleFinding;
use super::{CompiledOptions, RULE_ID};
use crate::codebase::ts_source::relative_slash_path;
use serde_yaml::Value;
use std::collections::HashMap;
use std::path::Path;

pub(super) fn check_file(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    crate::perf_trace::trace("github-actions-composite-step-schema.scan", || {
        let rel = relative_slash_path(root, path);
        let Some(source) = super::super::read_source(sources, path) else {
            return Vec::new();
        };
        check_source(&rel, &source, opts)
    })
}

fn check_source(rel: &str, source: &str, opts: &CompiledOptions) -> Vec<RuleFinding> {
    if crate::codebase::ts_source::has_disable_file_comment(source, RULE_ID) {
        return Vec::new();
    }
    let mut findings = match serde_yaml::from_str::<Value>(source) {
        Ok(value) => check_parsed(rel, source, &value, opts),
        Err(err) => vec![invalid_yaml_finding(rel, &err)],
    };
    super::super::suppress_rule_findings_with_source(&mut findings, source);
    findings
}

fn check_parsed(
    rel: &str,
    source: &str,
    value: &Value,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    let Some(runs) = value.get("runs").and_then(Value::as_mapping) else {
        return Vec::new();
    };
    match runs.get("using").and_then(Value::as_str) {
        Some("composite") => {}
        _ => return Vec::new(),
    }
    let Some(steps) = runs.get("steps").and_then(Value::as_sequence) else {
        return Vec::new();
    };

    let mut findings = Vec::new();
    let mut key_occurrences: HashMap<String, usize> = HashMap::new();
    for (index, step) in steps.iter().enumerate() {
        let Some(step) = step.as_mapping() else {
            continue;
        };
        let label = step_label(step, index);
        let mut keys: Vec<&str> = step.keys().filter_map(Value::as_str).collect();
        keys.sort_unstable();
        keys.dedup();
        for key in keys {
            if !is_forbidden_key(key, opts) {
                continue;
            }
            let occurrence = key_occurrences.entry(key.to_string()).or_insert(0);
            let line = mapping_key_line(source, key, *occurrence);
            *occurrence += 1;
            findings.push(unsupported_key_finding(rel, line, &label, key));
        }
    }
    findings
}

fn is_forbidden_key(key: &str, opts: &CompiledOptions) -> bool {
    opts.extra_forbidden_keys.contains(key) || !opts.allowed_keys.contains(key)
}

fn step_label(step: &serde_yaml::Mapping, index: usize) -> String {
    mapping_string(step, "name")
        .or_else(|| mapping_string(step, "id"))
        .or_else(|| mapping_string(step, "uses"))
        .unwrap_or_else(|| format!("step #{}", index + 1))
}

fn mapping_string(step: &serde_yaml::Mapping, key: &str) -> Option<String> {
    step.get(Value::String(key.to_string()))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn unsupported_key_finding(rel: &str, line: usize, label: &str, key: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line,
        message: format!(
            "composite action step \"{label}\" sets \"{key}\", which GitHub does not support on composite-action steps; remove it and set it on the calling workflow step instead."
        ),
        import: None,
        target: Some(key.to_string()),
    }
}

fn invalid_yaml_finding(rel: &str, err: &serde_yaml::Error) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line: yaml_parse_line(err),
        message: format!("{rel}: invalid YAML ({err})"),
        import: None,
        target: None,
    }
}

pub(crate) fn yaml_parse_line(err: &serde_yaml::Error) -> usize {
    err.location().map_or(1, |location| location.line())
}

pub(crate) fn mapping_key_line(source: &str, key: &str, occurrence: usize) -> usize {
    mapping_key_candidate_lines(source, key)
        .nth(occurrence)
        .unwrap_or(1)
}

/// Mapping-key lines that are not inside a `|` / `>` block scalar.
///
/// A block-scalar body can contain `timeout-minutes:` as documentation. Those
/// lines must not steal the occurrence index of a later real sibling key.
fn mapping_key_candidate_lines<'a>(
    source: &'a str,
    key: &'a str,
) -> impl Iterator<Item = usize> + 'a {
    let mut block_indent = None;
    source.lines().enumerate().filter_map(move |(index, line)| {
        let indent = leading_indent(line);
        let trimmed = line.trim();
        if let Some(key_indent) = block_indent {
            if trimmed.is_empty() || trimmed.starts_with('#') || indent > key_indent {
                return None;
            }
            block_indent = None;
        }
        if starts_block_scalar(line) {
            block_indent = Some(indent);
        }
        contains_mapping_key(line, key).then_some(index + 1)
    })
}

fn leading_indent(line: &str) -> usize {
    line.chars()
        .take_while(|ch| matches!(ch, ' ' | '\t'))
        .count()
}

pub(crate) fn starts_block_scalar(line: &str) -> bool {
    let comment_free = line.split('#').next().unwrap_or(line);
    let Some(colon) = comment_free.find(':') else {
        return false;
    };
    if comment_free[..colon].trim().is_empty() {
        return false;
    }
    is_block_indicator(comment_free[colon + 1..].trim())
}

fn is_block_indicator(value: &str) -> bool {
    let rest = match value.as_bytes().first() {
        Some(b'|' | b'>') => &value[1..],
        _ => return false,
    };
    let rest = rest
        .strip_prefix('+')
        .or_else(|| rest.strip_prefix('-'))
        .unwrap_or(rest);
    rest.trim_start_matches(|ch: char| ch.is_ascii_digit())
        .trim()
        .is_empty()
}

pub(crate) fn contains_mapping_key(line: &str, key: &str) -> bool {
    let comment_free = line.split('#').next().unwrap_or(line);
    let mut rest = comment_free;
    while let Some(idx) = rest.find(key) {
        let before_ok = idx == 0 || !is_key_char(rest.as_bytes()[idx - 1]);
        let after = &rest[idx + key.len()..];
        let after_ok = after.trim_start().starts_with(':')
            && after
                .chars()
                .next()
                .is_none_or(|ch| ch == ':' || ch.is_whitespace());
        if before_ok && after_ok {
            return true;
        }
        rest = &rest[idx + key.len()..];
    }
    false
}

fn is_key_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'
}
