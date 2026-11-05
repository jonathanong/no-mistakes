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
    let sources = super::source_store_for_files(all_files);
    check_with_files_and_sources(root, config, all_files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.try_rule_options()?;
            let files = super::selected_rule_files(root, config, rule, all_files)?;
            let config_file = super::configured_root::config_file_for_rule(root, config, rule);
            let audit = super::allow_audit::AllowAudit::new(
                root,
                config,
                rule,
                Some(sources),
                "allowlist",
                opts.allowlist.len(),
            );
            let scope = super::allow_audit::CheckedScope::new(root, Some(sources), &files);
            let mut findings = scan(root, &opts, &files, &config_file, &audit);
            findings.extend(audit.finish(
                |index| {
                    scope.exact(&opts.allowlist[index])
                        && audited(&opts.allowlist[index], &opts.scopes)
                },
                |index| opts.allowlist[index].clone(),
            ));
            Ok(findings)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    config_file: &str,
    audit: &super::allow_audit::AllowAudit,
) -> Vec<RuleFinding> {
    let allowlist = HashSet::new();
    let mut findings = escaping_scope_findings(opts, config_file);
    // `par_iter` is a rayon iterator. Collect it before extending the Vec.
    let file_findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file(path, root, &allowlist, &opts.scopes))
        .filter(|finding| {
            !audit
                .usage
                .suppress(opts.allowlist.iter().map(|entry| entry == &finding.file))
        })
        .collect();
    findings.extend(file_findings);
    findings.sort_by(|a, b| a.file.cmp(&b.file));
    findings
}

fn escaping_scope_findings(opts: &Options, config_file: &str) -> Vec<RuleFinding> {
    opts.scopes
        .iter()
        .filter(|scope| normalized_scope_path(&scope.path).is_none())
        .map(|scope| {
            let display = scope.path.replace('\\', "/");
            RuleFinding {
                rule: RULE_ID.to_string(),
                file: config_file.to_string(),
                line: 1,
                message: format!("{display}: configured root escapes the repository"),
                import: None,
                target: None,
            }
        })
        .collect()
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
        // `./src` and `src/../src` are the same prefix as `src`. An escaping
        // scope is one config finding from `scan`, not a hit on each file.
        let Some(scope_path) = normalized_scope_path(&scope.path) else {
            continue;
        };
        let in_scope =
            scope_path == "." || rel == scope_path || rel.starts_with(&format!("{scope_path}/"));
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

fn normalized_scope_path(path: &str) -> Option<String> {
    let normalized = crate::codebase::workflow_topology::posix_path::normalize(path);
    if normalized == ".." || normalized.starts_with("../") {
        None
    } else {
        Some(normalized)
    }
}

fn audited(file: &str, scopes: &[ScopeSpec]) -> bool {
    !is_typescript_declaration_file(file)
        && scopes.iter().any(|scope| {
            normalized_scope_path(&scope.path).is_some_and(|path| {
                path == "." || file == path || file.starts_with(&format!("{path}/"))
            })
        })
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
