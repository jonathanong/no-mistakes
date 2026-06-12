use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::codebase::workspaces;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "package-json-workspace-coverage";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) package_roots: Vec<String>,
    pub(crate) allowlist: Vec<String>,
    pub(crate) require_named_package: bool,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(root, &opts, &files)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    if opts.package_roots.is_empty() {
        return Ok(Vec::new());
    }

    let workspace_dirs: HashSet<String> = workspaces::load_from_files(root, files)?
        .packages
        .iter()
        .map(|pkg| relative_slash_path(root, &pkg.dir))
        .collect();
    let allowlist: HashSet<&str> = opts.allowlist.iter().map(String::as_str).collect();

    let mut findings = Vec::new();
    for path in files {
        if path.file_name().and_then(|name| name.to_str()) != Some("package.json") {
            continue;
        }
        let rel = relative_slash_path(root, path);
        if allowlist.contains(rel.as_str()) {
            continue;
        }
        let dir = path
            .parent()
            .expect("discovered package.json paths have a parent directory");
        let dir_rel = relative_slash_path(root, dir);
        if dir_rel.is_empty() || !path_under_package_roots(&dir_rel, &opts.package_roots) {
            continue;
        }
        if workspace_dirs.contains(&dir_rel) {
            continue;
        }
        if opts.require_named_package && package_name(path).is_none() {
            continue;
        }
        findings.push(RuleFinding {
            rule: RULE_ID.to_string(),
            file: rel.clone(),
            line: 1,
            message: format!("{rel}: package directory is not covered by the workspace config"),
            import: None,
            target: Some(dir_rel),
        });
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

fn path_under_package_roots(path: &str, package_roots: &[String]) -> bool {
    package_roots.iter().any(|root| {
        let root = root.trim_matches('/');
        !root.is_empty() && (path == root || path.starts_with(&format!("{root}/")))
    })
}

fn package_name(path: &Path) -> Option<String> {
    let source = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&source).ok()?;
    json.get("name")?.as_str().map(str::to_string)
}

#[cfg(test)]
mod tests;
