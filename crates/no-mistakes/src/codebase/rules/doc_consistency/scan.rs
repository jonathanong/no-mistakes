use super::super::RuleFinding;
use super::options::Options;
use super::patterns::{self, CompiledPattern};
use super::RULE_ID;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let sources = super::super::source_store_for_files(files);
    scan_with_sources(root, opts, files, &sources)
}

pub(super) fn scan_with_sources(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    // Reject invalid patterns before any source read, including an empty selection.
    let compiled = patterns::compile(&opts.banned_patterns)?;
    let rel_set: HashSet<String> = files
        .iter()
        .map(|path| relative_slash_path(root, path))
        .collect();
    let mut findings = required_findings(root, opts, &rel_set, sources);
    if !opts.banned_substrings.is_empty() || !compiled.is_empty() {
        findings.extend(ban_findings(root, opts, &compiled, files, sources));
    }
    findings.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then(left.line.cmp(&right.line))
            .then(left.message.cmp(&right.message))
    });
    Ok(findings)
}

fn required_findings(
    root: &Path,
    opts: &Options,
    rel_set: &HashSet<String>,
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    opts.required_files
        .par_iter()
        .flat_map(|req_file| required_file_findings(root, opts, rel_set, sources, req_file))
        .collect()
}

fn required_file_findings(
    root: &Path,
    opts: &Options,
    rel_set: &HashSet<String>,
    sources: &SourceStore,
    req_file: &str,
) -> Vec<RuleFinding> {
    if !rel_set.contains(req_file) {
        return vec![missing_file(req_file)];
    }
    let Some(content) = super::super::read_source(sources, &root.join(req_file)) else {
        return Vec::new();
    };
    let mut local = Vec::new();
    if let Some(heading) = &opts.required_heading {
        let heading_text = heading.trim_start_matches('#').trim();
        if !crate::codebase::markdown_sections::has_section(&content, heading_text) {
            local.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: req_file.to_string(),
                line: 1,
                message: format!("{req_file}: missing required heading \"{heading}\""),
                import: None,
                target: None,
            });
        }
    }
    for spec in &opts.required_substrings {
        if spec.file != req_file || content.contains(spec.substring.as_str()) {
            continue;
        }
        local.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: req_file.to_string(),
            line: 1,
            message: format!(
                "{req_file}: missing required substring \"{}\"",
                spec.substring
            ),
            import: None,
            target: None,
        });
    }
    local
}

fn missing_file(req_file: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: req_file.to_string(),
        line: 1,
        message: "file not found or untracked".to_string(),
        import: None,
        target: None,
    }
}

fn ban_findings(
    root: &Path,
    opts: &Options,
    compiled: &[CompiledPattern],
    files: &[PathBuf],
    sources: &SourceStore,
) -> Vec<RuleFinding> {
    files
        .par_iter()
        .flat_map(|path| {
            let Some(content) = super::super::read_source(sources, path) else {
                return Vec::new();
            };
            let rel = relative_slash_path(root, path);
            let mut local = Vec::new();
            for banned in &opts.banned_substrings {
                if content.contains(banned.as_str()) {
                    local.push(RuleFinding {
                        rule: RULE_ID.to_string(),
                        file: rel.clone(),
                        line: 1,
                        message: format!("{rel}: contains banned substring \"{banned}\""),
                        import: None,
                        target: None,
                    });
                }
            }
            if !compiled.is_empty() {
                local.extend(patterns::findings_in_file(&rel, &content, compiled));
            }
            local
        })
        .collect()
}
