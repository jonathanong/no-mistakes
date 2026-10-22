use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod yaml_uses;

pub const RULE_ID: &str = "github-actions-pinned-hash";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) exclude_paths: Vec<String>,
}

super::sourced_file_rule_entrypoints!();

pub(crate) fn build_exclude_globset(patterns: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for pat in patterns {
        if let Ok(glob) = Glob::new(pat) {
            builder.add(glob);
        }
    }
    builder.build().unwrap_or_else(|_| GlobSet::empty())
}

pub(crate) fn build_patterns() -> (Regex, Regex, Regex) {
    (
        Regex::new(r"^\s*-?\s*uses:\s+(\S+)(.*)$").expect("uses pattern"),
        Regex::new(r"^[0-9a-fA-F]{40}$").expect("sha pattern"),
        // Allow `# v1`, `# v1.2`, `# v1.2.3`, `# 1.87.0`, `# v1.0.0-beta+build.1`
        Regex::new(r"^#\s*v?\d+(?:\.\d+)*(?:[-+][\w.+-]+)?\s*$").expect("version comment pattern"),
    )
}

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let exclude_set = build_exclude_globset(&opts.exclude_paths);
    let (uses_re, sha_re, version_re) = build_patterns();
    let findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| {
            check_file_with_sources(
                path,
                root,
                &exclude_set,
                &uses_re,
                &sha_re,
                &version_re,
                sources,
            )
        })
        .collect();
    Ok(findings)
}

fn check_file_with_sources(
    path: &Path,
    root: &Path,
    exclude_set: &GlobSet,
    uses_re: &Regex,
    sha_re: &Regex,
    version_re: &Regex,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    if exclude_set.is_match(rel.as_str()) {
        return Vec::new();
    }
    let ext = path.extension().and_then(|extension| extension.to_str());
    if !matches!(ext, Some("yml" | "yaml"))
        || (!rel.starts_with(".github/workflows/")
            && !(rel.starts_with(".github/actions/")
                && (rel.ends_with("/action.yml") || rel.ends_with("/action.yaml"))))
    {
        return Vec::new();
    }
    let Some(content) = super::read_source(sources, path) else {
        return Vec::new();
    };
    check_source(path, root, uses_re, sha_re, version_re, &content, sources)
}

fn check_source(
    path: &Path,
    root: &Path,
    uses_re: &Regex,
    sha_re: &Regex,
    version_re: &Regex,
    content: &str,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let file = relative_slash_path(root, path);
    let mut findings = Vec::new();
    // Lines the regex already accepted or flagged. Parsed `uses` values must
    // not report those lines again.
    let mut seen_lines = BTreeSet::new();

    for (i, line) in content.lines().enumerate() {
        let line_num = i + 1;
        if !line.contains("uses:") {
            continue;
        }
        let Some(caps) = uses_re.captures(line) else {
            continue;
        };
        let raw = caps.get(1).map_or("", |m| m.as_str());
        // Anchors, aliases, and block-scalar headers are resolved by the parsed walk.
        if raw.starts_with('&') || raw.starts_with('*') || yaml_uses::is_block_header(raw) {
            continue;
        }
        seen_lines.insert(line_num);
        let uses_value = raw
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .or_else(|| raw.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
            .unwrap_or(raw);
        let trailing = caps.get(2).map_or("", |m| m.as_str()).trim();

        if yaml_uses::exempt(uses_value) {
            continue;
        }
        if let Some(finding) =
            yaml_uses::unpinned_finding(&file, line_num, uses_value, sha_re, version_re, trailing)
        {
            findings.push(finding);
        }
    }

    // Parse failures keep the line-scan findings. Each document that parses
    // is walked, including a flow `uses` in a later document.
    let documents = sources.parse_yaml_documents_path(path).ok();
    yaml_uses::append_parsed_uses(
        content,
        documents.as_deref().unwrap_or(&[]),
        &file,
        sha_re,
        version_re,
        &seen_lines,
        &mut findings,
    );
    findings.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then(left.message.cmp(&right.message))
    });
    findings
}

#[cfg(test)]
mod tests;
