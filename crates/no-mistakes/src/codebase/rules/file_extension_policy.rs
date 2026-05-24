use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "file-extension-policy";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct ScopeSpec {
    pub(crate) path: String,
    pub(crate) banned_extensions: Vec<String>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) allowlist: Vec<String>,
    pub(crate) scopes: Vec<ScopeSpec>,
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
    let allowlist: HashSet<&str> = opts.allowlist.iter().map(String::as_str).collect();
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file(path, root, &allowlist, &opts.scopes))
        .collect();
    findings.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(findings)
}

pub(crate) fn check_file(
    path: &Path,
    root: &Path,
    allowlist: &HashSet<&str>,
    scopes: &[ScopeSpec],
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);

    if allowlist.contains(rel.as_str()) {
        return Vec::new();
    }

    // Skip TypeScript declaration files
    if rel.ends_with(".d.ts") {
        return Vec::new();
    }

    let ext = file_extension(&rel);

    let mut findings = Vec::new();
    for scope in scopes {
        let scope_path = scope.path.trim_end_matches('/');
        let in_scope = rel == scope_path || rel.starts_with(&format!("{scope_path}/"));
        if !in_scope {
            continue;
        }
        if scope
            .banned_extensions
            .iter()
            .any(|banned| banned.as_str() == ext)
        {
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.clone(),
                line: 1,
                message: format!("{rel}: file uses {ext} extension — align with scope policy"),
                import: None,
                target: None,
            });
        }
    }
    findings
}

fn file_extension(rel: &str) -> &str {
    match rel.rfind('.') {
        Some(i) => &rel[i..],
        None => "",
    }
}

#[cfg(test)]
mod tests;
