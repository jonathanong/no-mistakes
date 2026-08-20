use super::parse::{key_line, parse_source, pin_kind, read_text, tracked_rels, value_at_key};
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
) -> Vec<RuleFinding> {
    if opts.source_file.is_empty() || opts.anchors.is_empty() {
        return Vec::new();
    }
    let tracked = tracked_rels(root, files);
    let relevant: Vec<&str> = std::iter::once(opts.source_file.as_str())
        .chain(opts.anchors.iter().map(|anchor| anchor.file.as_str()))
        .filter(|rel| !rel.is_empty())
        .collect();
    if relevant.iter().all(|rel| !tracked.contains(*rel)) {
        return Vec::new();
    }
    let source_text = read_text(root, &opts.source_file, sources);
    let parsed = match parse_source(Path::new(&opts.source_file), &source_text) {
        Ok(value) => value,
        Err(error) => {
            return vec![finding(
                &opts.source_file,
                1,
                format!("{}: {error}", opts.source_file),
            )];
        }
    };
    let mut findings = Vec::new();
    for anchor in &opts.anchors {
        findings.extend(check_anchor(
            root,
            opts,
            &parsed,
            &source_text,
            anchor,
            sources,
        ));
    }
    super::super::suppress_rule_findings_with_sources(root, &mut findings, sources);
    findings
}

fn check_anchor(
    root: &Path,
    opts: &Options,
    parsed: &Value,
    source_text: &str,
    anchor: &Anchor,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    if anchor.file.is_empty() {
        return Vec::new();
    }
    let label = if anchor.label.is_empty() {
        anchor.file.as_str()
    } else {
        anchor.label.as_str()
    };
    let regex = match compile_pattern(&anchor.pattern, &opts.source_file, label) {
        Ok(regex) => regex,
        Err(message) => return vec![finding(&opts.source_file, 1, message)],
    };
    let line = key_line(source_text, &opts.source_key);
    let pin = match value_at_key(parsed, &opts.source_key) {
        None => {
            return vec![finding(
                &opts.source_file,
                line,
                format!(
                    "{}: expected key \"{}\" for {label} not found",
                    opts.source_file, opts.source_key
                ),
            )];
        }
        Some(Value::String(pin)) => pin.as_str(),
        Some(value) => {
            return vec![finding(
                &opts.source_file,
                line,
                format!(
                    "{}: expected key \"{}\" for {label} to be a string pin, got {} — invalid pin",
                    opts.source_file,
                    opts.source_key,
                    pin_kind(value)
                ),
            )];
        }
    };
    let anchor_text = read_text(root, &anchor.file, sources);
    let Some(captures) = regex.captures(&anchor_text) else {
        return vec![finding(
            &anchor.file,
            1,
            format!(
                "{}: could not find {label} version reference matching expected pattern",
                anchor.file
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
    vec![finding(
        &anchor.file,
        line,
        format!(
            "{}: {label} version mismatch — {} says \"{captured}\" but {} pins \"{pin}\". \
             Update both in the same commit.",
            anchor.file, anchor.file, opts.source_file
        ),
    )]
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
