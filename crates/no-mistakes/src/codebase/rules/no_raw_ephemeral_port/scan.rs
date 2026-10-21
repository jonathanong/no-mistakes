use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::ts_source::relative_slash_path;
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn check_file(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    check_file_with_prefilter(root, path, opts, sources, defer_suppression, true)
}

pub(super) fn check_file_with_prefilter(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
    prefilter: bool,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    if opts.allow.is_match(&rel) {
        return Vec::new();
    }
    let Some(source) = super::super::read_source(sources, path) else {
        return Vec::new();
    };
    if !defer_suppression && crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID)
    {
        return Vec::new();
    }
    collect_port_findings(path, &rel, &source, opts, defer_suppression, prefilter)
}

fn collect_port_findings(
    path: &Path,
    rel: &str,
    source: &str,
    opts: &CompiledOptions,
    defer_suppression: bool,
    prefilter: bool,
) -> Vec<RuleFinding> {
    let mut seen = BTreeSet::new();
    let mut findings = Vec::new();
    let bytes = source.as_bytes();
    if is_tuple_bind_source(path)
        && (!prefilter || super::super::byte_prefilter::contains_bytes(bytes, b"bind"))
    {
        for line in super::python::scan_lines(source, &opts.bind) {
            push_finding(&mut findings, &mut seen, rel, line, &opts.message);
        }
    }
    if is_js_ts(path)
        && (!prefilter
            || super::super::byte_prefilter::contains_bytes(bytes, b"listen")
            || super::super::byte_prefilter::contains_bytes(bytes, b"\\"))
    {
        for line in super::ast::scan_lines(path, source) {
            push_finding(&mut findings, &mut seen, rel, line, &opts.message);
        }
    }
    if !defer_suppression {
        super::super::suppress_rule_findings_with_source(&mut findings, source);
    }
    findings
}

fn push_finding(
    findings: &mut Vec<RuleFinding>,
    seen: &mut BTreeSet<usize>,
    rel: &str,
    line: usize,
    message: &str,
) {
    if !seen.insert(line) {
        return;
    }
    findings.push(RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line,
        message: message.to_string(),
        import: None,
        target: None,
    });
}

fn is_tuple_bind_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("py" | "sh" | "bash" | "zsh" | "yml" | "yaml")
    )
}

fn is_js_ts(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("js" | "ts" | "mjs" | "mts" | "cjs" | "cts" | "tsx" | "jsx")
    )
}
