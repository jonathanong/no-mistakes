use super::{path_filter::GlobMatcher, RuleFinding};
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path, SourceStore};
use std::path::Path;

pub(crate) struct MaskedFileContext<'a> {
    pub(crate) root: &'a Path,
    pub(crate) path: &'a Path,
    pub(crate) rule_id: &'a str,
    pub(crate) allow: &'a GlobMatcher,
    pub(crate) sources: &'a SourceStore,
    pub(crate) defer_suppression: bool,
}

impl<'a> MaskedFileContext<'a> {
    pub(crate) fn new(
        root: &'a Path,
        path: &'a Path,
        rule_id: &'a str,
        allow: &'a GlobMatcher,
        sources: &'a SourceStore,
        defer_suppression: bool,
    ) -> Self {
        Self {
            root,
            path,
            rule_id,
            allow,
            sources,
            defer_suppression,
        }
    }
}

pub(crate) fn scan_masked_files<T>(
    root: &Path,
    opts: &T,
    files: &[std::path::PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
    check: impl Fn(&Path, &Path, &T, &SourceStore, bool) -> Vec<RuleFinding>,
) -> Vec<RuleFinding> {
    files
        .iter()
        .flat_map(|path| check(root, path, opts, sources, defer_suppression))
        .collect()
}

/// Read, mask, and apply suppression once for file-local Swift/C# scans.
pub(crate) fn scan_masked_file(
    context: MaskedFileContext<'_>,
    mask: fn(&str) -> String,
    detect: impl FnOnce(&str, &str, &str) -> Vec<RuleFinding>,
) -> Vec<RuleFinding> {
    let MaskedFileContext {
        root,
        path,
        rule_id,
        allow,
        sources,
        defer_suppression,
    } = context;
    let rel = relative_slash_path(root, path);
    if allow.is_match(&rel) {
        return Vec::new();
    }
    let Some(source) = super::read_source(sources, path) else {
        return Vec::new();
    };
    if !defer_suppression && has_disable_file_comment(&source, rule_id) {
        return Vec::new();
    }
    let code = mask(&source);
    let mut findings = detect(&source, &code, &rel);
    if !defer_suppression {
        super::suppress_rule_findings_with_source(&mut findings, &source);
    }
    findings
}
