use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::ts_source::relative_slash_path;
use std::cell::Cell;
use std::collections::BTreeSet;
use std::path::Path;

thread_local! {
    pub(super) static BIND_PREFILTER_SKIPS: Cell<u32> = const { Cell::new(0) };
    pub(super) static LISTEN_PREFILTER_SKIPS: Cell<u32> = const { Cell::new(0) };
}

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
    if !is_tuple_bind_source(path) && !is_js_ts(path) {
        return Vec::new();
    }
    let source = match super::super::source_access::checked_source(sources, path, root, RULE_ID) {
        Ok(source) => source,
        Err(finding) => {
            return if opts.allow.is_match(&rel) {
                Vec::new()
            } else {
                vec![*finding]
            }
        }
    };
    let (mut findings, checked) = collect_port_findings(path, &rel, &source, opts, prefilter);
    if checked {
        opts.allow.checked(path);
    }
    if opts.allow.suppress(&rel, !findings.is_empty())
        || (!defer_suppression
            && crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID))
    {
        return Vec::new();
    }
    if !defer_suppression {
        super::super::suppress_rule_findings_with_source(&mut findings, &source);
    }
    findings
}

fn collect_port_findings(
    path: &Path,
    rel: &str,
    source: &str,
    opts: &CompiledOptions,
    prefilter: bool,
) -> (Vec<RuleFinding>, bool) {
    let mut checked = true;
    let mut seen = BTreeSet::new();
    let mut findings = Vec::new();
    let bytes = source.as_bytes();
    if is_tuple_bind_source(path) {
        if prefilter && !super::super::byte_prefilter::contains_bytes(bytes, b"bind") {
            BIND_PREFILTER_SKIPS.with(|skips| skips.set(skips.get() + 1));
        } else {
            for line in super::python::scan_lines(source, &opts.bind) {
                push_finding(&mut findings, &mut seen, rel, line, &opts.message);
            }
        }
    }
    if is_js_ts(path) {
        if prefilter
            && !super::super::byte_prefilter::contains_bytes(bytes, b"listen")
            && !super::super::byte_prefilter::contains_bytes(bytes, b"\\")
        {
            LISTEN_PREFILTER_SKIPS.with(|skips| skips.set(skips.get() + 1));
        } else {
            let (lines, parsed) = super::ast::scan_lines_checked(path, source);
            checked = parsed;
            for line in lines {
                push_finding(&mut findings, &mut seen, rel, line, &opts.message);
            }
        }
    }
    (findings, checked)
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
        causes: Vec::new(),
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
