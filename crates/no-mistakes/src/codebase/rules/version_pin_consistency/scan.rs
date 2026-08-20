use super::parse::{configured_rel, key_line, parse_source, pin_kind, read_text, tracked_rels};
use super::{Anchor, Options, RuleFinding, RULE_ID};
use crate::codebase::ts_source::line_number;
use regex::Regex;
use serde_yaml::Value;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    if opts.source_file.is_empty() || opts.anchors.is_empty() {
        return Vec::new();
    }
    let tracked = tracked_rels(root, files);
    let source_rel = configured_rel(&opts.source_file);
    let source_tracked = tracked.contains(source_rel);
    let remaining: Vec<&Anchor> = opts
        .anchors
        .iter()
        .filter(|anchor| !anchor.file.is_empty() && tracked.contains(configured_rel(&anchor.file)))
        .collect();
    if !source_tracked && remaining.is_empty() {
        return Vec::new();
    }
    let source_text = read_text(root, source_rel, sources);
    let parsed = match parse_source(Path::new(source_rel), &source_text) {
        Ok(value) => value,
        Err(error) => {
            return finish(
                root,
                sources,
                defer_suppression,
                source_finding(
                    source_tracked,
                    source_rel,
                    1,
                    format!("{source_rel}: {error}"),
                ),
            );
        }
    };
    let mut findings = Vec::new();
    let source = SourcePin {
        rel: source_rel,
        tracked: source_tracked,
        text: &source_text,
        parsed: &parsed,
    };
    for anchor in remaining {
        findings.extend(check_anchor(root, opts, source, anchor, sources));
    }
    finish(root, sources, defer_suppression, findings)
}

#[derive(Clone, Copy)]
struct SourcePin<'a> {
    rel: &'a str,
    tracked: bool,
    text: &'a str,
    parsed: &'a Value,
}

fn check_anchor(
    root: &Path,
    opts: &Options,
    source: SourcePin<'_>,
    anchor: &Anchor,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let anchor_rel = configured_rel(&anchor.file);
    let label = if anchor.label.is_empty() {
        anchor_rel
    } else {
        anchor.label.as_str()
    };
    let regex = match compile_pattern(&anchor.pattern, source.rel, label) {
        Ok(regex) => regex,
        Err(message) => return source_finding(source.tracked, source.rel, 1, message),
    };
    let line = key_line(source.text, &opts.source_key);
    let pin = match value_pin(source.parsed, opts, source.rel, label) {
        Ok(pin) => pin,
        Err(message) => return source_finding(source.tracked, source.rel, line, message),
    };
    let anchor_text = read_text(root, anchor_rel, sources);
    let Some(captures) = regex.captures(&anchor_text) else {
        return vec![finding(
            anchor_rel,
            1,
            format!(
                "{anchor_rel}: could not find {label} version reference matching expected pattern"
            ),
        )];
    };
    let captured = captures.get(1).map_or("", |m| m.as_str());
    if captured == pin {
        return Vec::new();
    }
    let line = captures
        .get(1)
        .map_or(1, |m| line_number(&anchor_text, m.start() as u32));
    let source_rel = source.rel;
    vec![finding(
        anchor_rel,
        line,
        format!(
            "{anchor_rel}: {label} version mismatch — {anchor_rel} says \"{captured}\" but {source_rel} pins \"{pin}\". \
             Update both in the same commit."
        ),
    )]
}

fn value_pin<'a>(
    parsed: &'a Value,
    opts: &Options,
    source_rel: &str,
    label: &str,
) -> Result<&'a str, String> {
    match super::parse::value_at_key(parsed, &opts.source_key) {
        None => Err(format!(
            "{source_rel}: expected key \"{}\" for {label} not found",
            opts.source_key
        )),
        Some(Value::String(pin)) => Ok(pin.as_str()),
        Some(value) => Err(format!(
            "{source_rel}: expected key \"{}\" for {label} to be a string pin, got {} — invalid pin",
            opts.source_key,
            pin_kind(value)
        )),
    }
}

fn compile_pattern(pattern: &str, source_file: &str, label: &str) -> Result<Regex, String> {
    let regex = Regex::new(pattern)
        .map_err(|error| format!("{source_file}: {label} pattern is invalid ({error})"))?;
    if regex.captures_len() != 2 {
        return Err(format!(
            "{source_file}: {label} pattern must have exactly one capturing group"
        ));
    }
    Ok(regex)
}

fn source_finding(tracked: bool, file: &str, line: usize, message: String) -> Vec<RuleFinding> {
    if tracked {
        vec![finding(file, line, message)]
    } else {
        Vec::new()
    }
}

fn finish(
    root: &Path,
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
    mut findings: Vec<RuleFinding>,
) -> Vec<RuleFinding> {
    if !defer_suppression {
        super::super::suppress_rule_findings_with_sources(root, &mut findings, sources);
    }
    findings
}

fn finding(file: &str, line: usize, message: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message,
        import: None,
        target: None,
    }
}
