use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
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
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        let target_roots = super::target_roots(root, config, rule);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|p| target_roots.iter().any(|r| p.starts_with(r)))
            .cloned()
            .collect();
        findings.extend(scan(root, &opts, &files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    // Build a set of relative paths for existence checking
    let rel_set: HashSet<String> = files.iter().map(|p| relative_slash_path(root, p)).collect();

    let mut findings = Vec::new();

    // Check required files exist
    for req_file in &opts.required_files {
        if !rel_set.contains(req_file.as_str()) {
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: req_file.clone(),
                line: 1,
                message: "file not found or untracked".to_string(),
                import: None,
                target: None,
            });
            continue;
        }

        // File exists — read and check content
        let abs_path = root.join(req_file);
        let Ok(content) = std::fs::read_to_string(&abs_path) else {
            continue;
        };

        // Check required heading
        if let Some(heading) = &opts.required_heading {
            let heading_text = heading.trim_start_matches('#').trim();
            if !crate::codebase::markdown_sections::has_section(&content, heading_text) {
                findings.push(RuleFinding {
                    rule: RULE_ID.to_string(),
                    file: req_file.clone(),
                    line: 1,
                    message: format!("{req_file}: missing required heading \"{heading}\""),
                    import: None,
                    target: None,
                });
            }
        }

        // Check required substrings for this file
        for spec in &opts.required_substrings {
            if spec.file != *req_file {
                continue;
            }
            if !content.contains(spec.substring.as_str()) {
                findings.push(RuleFinding {
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
    }

    // Check banned substrings across all tracked files (read each file once)
    for path in files {
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        let rel = relative_slash_path(root, path);
        for banned in &opts.banned_substrings {
            if content.contains(banned.as_str()) {
                findings.push(RuleFinding {
                    rule: RULE_ID.to_string(),
                    file: rel.clone(),
                    line: 1,
                    message: format!("{rel}: contains banned substring \"{banned}\""),
                    import: None,
                    target: None,
                });
            }
        }
    }

    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

#[cfg(test)]
mod tests;
