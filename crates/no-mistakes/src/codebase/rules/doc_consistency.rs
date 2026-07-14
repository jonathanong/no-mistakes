use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "doc-consistency";

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct SubstringSpec {
    pub(crate) file: String,
    pub(crate) substring: String,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) required_files: Vec<String>,
    pub(crate) required_heading: Option<String>,
    pub(crate) required_substrings: Vec<SubstringSpec>,
    pub(crate) banned_substrings: Vec<String>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let skip = &config.filesystem.skip_directories;
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        let target_roots = super::target_roots(root, config, rule);
        let files: Vec<PathBuf> = target_roots
            .iter()
            .flat_map(|r| discover_files(r, skip))
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan(root, &opts, &files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        let target_roots = super::target_roots(root, config, rule);
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
        findings.extend(scan_with_sources(root, &opts, &files, sources)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(files);
    scan_with_sources(root, opts, files, &sources)
}

fn scan_with_sources(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let rel_set: HashSet<String> = files.iter().map(|p| relative_slash_path(root, p)).collect();

    // Check required files (parallel)
    let mut findings: Vec<RuleFinding> = opts
        .required_files
        .par_iter()
        .flat_map(|req_file| {
            let mut local = Vec::new();
            if !rel_set.contains(req_file.as_str()) {
                local.push(RuleFinding {
                    rule: RULE_ID.to_string(),
                    file: req_file.clone(),
                    line: 1,
                    message: "file not found or untracked".to_string(),
                    import: None,
                    target: None,
                });
                return local;
            }
            let Some(content) = super::read_source(sources, &root.join(req_file)) else {
                return local;
            };
            if let Some(heading) = &opts.required_heading {
                let heading_text = heading.trim_start_matches('#').trim();
                if !crate::codebase::markdown_sections::has_section(&content, heading_text) {
                    local.push(RuleFinding {
                        rule: RULE_ID.to_string(),
                        file: req_file.clone(),
                        line: 1,
                        message: format!("{req_file}: missing required heading \"{heading}\""),
                        import: None,
                        target: None,
                    });
                }
            }
            for spec in &opts.required_substrings {
                if spec.file != *req_file {
                    continue;
                }
                if !content.contains(spec.substring.as_str()) {
                    local.push(RuleFinding {
                        rule: RULE_ID.to_string(),
                        file: req_file.clone(),
                        line: 1,
                        message: format!(
                            "{req_file}: missing required substring \"{}\"",
                            spec.substring
                        ),
                        import: None,
                        target: None,
                    });
                }
            }
            local
        })
        .collect();

    // Check banned substrings across all tracked files (parallel)
    if !opts.banned_substrings.is_empty() {
        let banned_findings: Vec<RuleFinding> = files
            .par_iter()
            .flat_map(|path| {
                let Some(content) = super::read_source(sources, path) else {
                    return Vec::new();
                };
                let rel = relative_slash_path(root, path);
                opts.banned_substrings
                    .iter()
                    .filter(|banned| content.contains(banned.as_str()))
                    .map(|banned| RuleFinding {
                        rule: RULE_ID.to_string(),
                        file: rel.clone(),
                        line: 1,
                        message: format!("{rel}: contains banned substring \"{banned}\""),
                        import: None,
                        target: None,
                    })
                    .collect()
            })
            .collect();
        findings.extend(banned_findings);
    }

    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

#[cfg(test)]
mod tests;
