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
    let files = discover_files(root, &config.filesystem.skip_directories);
    check_with_files(root, config, &files)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let mut findings: Vec<RuleFinding> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .flat_map(|rule| {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| target_roots.iter().any(|r| p.starts_with(r)))
                .cloned()
                .collect();
            scan(root, &opts, &files)
        })
        .collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Vec<RuleFinding> {
    let allowlist: HashSet<&str> = opts.allowlist.iter().map(String::as_str).collect();
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file(path, root, &allowlist, &opts.scopes))
        .collect();
    findings.sort_by(|a, b| a.file.cmp(&b.file));
    findings
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
    if is_typescript_declaration_file(&rel) {
        return Vec::new();
    }

    let ext = file_extension(&rel);

    for scope in scopes {
        let scope_path = scope.path.trim_end_matches('/');
        let in_scope = scope_path.is_empty()
            || scope_path == "."
            || rel == scope_path
            || rel.starts_with(&format!("{scope_path}/"));
        if !in_scope {
            continue;
        }
        if scope
            .banned_extensions
            .iter()
            .any(|banned| banned.as_str() == ext)
        {
            return vec![RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.clone(),
                line: 1,
                message: format!("{rel}: file uses {ext} extension — align with scope policy"),
                import: None,
                target: None,
            }];
        }
    }
    Vec::new()
}

fn file_extension(rel: &str) -> &str {
    match rel.rfind('.') {
        Some(i) => &rel[i..],
        None => "",
    }
}

fn is_typescript_declaration_file(rel: &str) -> bool {
    rel.ends_with(".d.ts") || rel.ends_with(".d.mts") || rel.ends_with(".d.cts")
}

#[cfg(test)]
mod tests;
