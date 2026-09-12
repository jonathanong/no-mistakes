use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path, SourceStore};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static CLASS_DECL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bclass\s+([A-Za-z_][A-Za-z0-9_]*)\b").expect("class pattern"));

const MODIFIERS: &[&str] = &[
    "public",
    "private",
    "internal",
    "open",
    "fileprivate",
    "final",
    "dynamic",
    "isolated",
    "nonisolated",
    "indirect",
    "distributed",
    "package",
];

#[inline(never)]
pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    files
        .iter()
        .flat_map(|path| check_file(root, path, opts, sources, defer_suppression))
        .collect()
}

#[inline(never)]
fn check_file(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &SourceStore,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    if opts.allow.is_match(&rel) {
        return Vec::new();
    }
    let Some(source) = super::super::read_source(sources, path) else {
        return Vec::new();
    };
    if !defer_suppression && has_disable_file_comment(&source, RULE_ID) {
        return Vec::new();
    }
    let code = super::super::lexical_mask::swift_code_mask(&source);
    let mut findings = Vec::new();
    for cap in CLASS_DECL.captures_iter(&code) {
        let Some(name) = cap.get(1) else { continue };
        if !name.as_str().ends_with(&opts.suffix) {
            continue;
        }
        let start = cap.get(0).map_or(name.start(), |m| m.start());
        if has_attribute(&code, start, &opts.attribute) {
            continue;
        }
        let line = source[..start].bytes().filter(|&b| b == b'\n').count() + 1;
        findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: rel.to_string(),
            line,
            message: opts.message.clone(),
            import: None,
            target: None,
        });
    }
    if !defer_suppression {
        super::super::suppress_rule_findings_with_source(&mut findings, &source);
    }
    findings
}

#[inline(never)]
fn has_attribute(source: &str, mut i: usize, attr: &str) -> bool {
    let bytes = source.as_bytes();
    loop {
        i = skip_ws(bytes, i);
        if i == 0 {
            return false;
        }
        if bytes[i - 1] == b')' {
            let Some(open) = matching_open_paren(bytes, i) else {
                return false;
            };
            i = skip_ws(bytes, open);
            let Some((ident, start)) = ident_ending_at(source, i) else {
                return false;
            };
            if start > 0 && bytes[start - 1] == b'@' {
                if ident == attr {
                    return true;
                }
                i = start - 1;
                continue;
            }
            return false;
        }
        let Some((ident, start)) = ident_ending_at(source, i) else {
            return false;
        };
        if start > 0 && bytes[start - 1] == b'@' {
            if ident == attr {
                return true;
            }
            i = start - 1;
            continue;
        }
        if MODIFIERS.contains(&ident) {
            i = start;
            continue;
        }
        return false;
    }
}

#[inline(never)]
fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while i > 0 && matches!(bytes[i - 1], b' ' | b'\t' | b'\n' | b'\r') {
        i -= 1;
    }
    i
}

#[inline(never)]
fn matching_open_paren(bytes: &[u8], mut i: usize) -> Option<usize> {
    let mut depth = 0;
    while i > 0 {
        i -= 1;
        match bytes[i] {
            b')' => depth += 1,
            b'(' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

#[inline(never)]
fn ident_ending_at(source: &str, end: usize) -> Option<(&str, usize)> {
    let bytes = source.as_bytes();
    if end == 0 || !is_ident(bytes[end - 1]) {
        return None;
    }
    let mut start = end - 1;
    while start > 0 && is_ident(bytes[start - 1]) {
        start -= 1;
    }
    Some((&source[start..end], start))
}

#[inline(never)]
fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}
