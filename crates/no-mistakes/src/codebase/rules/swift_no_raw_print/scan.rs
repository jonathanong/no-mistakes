use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path, SourceStore};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

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
    let mut seen = BTreeSet::new();
    let mut findings = Vec::new();
    for mat in opts.print.find_iter(&source) {
        if commented(&source, mat.start()) {
            continue;
        }
        let line = source[..mat.start()]
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
            + 1;
        if !seen.insert(line) {
            continue;
        }
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

fn commented(source: &str, start: usize) -> bool {
    let line_start = source[..start].rfind('\n').map_or(0, |i| i + 1);
    let prefix = source[line_start..start].trim_start();
    prefix.starts_with("//") || prefix.contains("//")
}
