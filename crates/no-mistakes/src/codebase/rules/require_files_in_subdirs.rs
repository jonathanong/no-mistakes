use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::Glob;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "require-files-in-subdirs";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct PackageSpec {
    pub(crate) root: PathBuf,
    pub(crate) required_files: Vec<String>,
    pub(crate) require_any_of: Vec<Vec<String>>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) packages: Vec<PackageSpec>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let skip = &config.filesystem.skip_directories;
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        if opts.packages.is_empty() {
            continue;
        }
        let files = discover_files(root, skip);
        findings.extend(scan(root, &opts, &files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        if opts.packages.is_empty() {
            continue;
        }
        findings.extend(scan(root, &opts, files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let file_set: HashSet<&PathBuf> = files.iter().collect();
    let mut findings = Vec::new();

    for spec in &opts.packages {
        let abs_root = if spec.root.is_absolute() {
            spec.root.clone()
        } else {
            root.join(&spec.root)
        };
        let subdirs = first_level_subdirs(&abs_root, files);
        for subdir in subdirs {
            check_subdir(root, &subdir, spec, files, &file_set, &mut findings)?;
        }
    }
    Ok(findings)
}

fn check_subdir(
    root: &Path,
    subdir: &Path,
    spec: &PackageSpec,
    files: &[PathBuf],
    file_set: &HashSet<&PathBuf>,
    findings: &mut Vec<RuleFinding>,
) -> Result<()> {
    let rel = relative_slash_path(root, subdir);

    for required in &spec.required_files {
        let candidate = subdir.join(required);
        if !file_set.contains(&candidate) {
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.clone(),
                line: 1,
                message: format!("{rel}: missing required file: {required}"),
                import: None,
                target: None,
            });
        }
    }

    for group in &spec.require_any_of {
        let any_match = group_matches(subdir, group, files)?;
        if !any_match {
            let group_str = group.join(", ");
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.clone(),
                line: 1,
                message: format!("{rel}: missing required file: {group_str}"),
                import: None,
                target: None,
            });
        }
    }
    Ok(())
}

fn group_matches(subdir: &Path, group: &[String], files: &[PathBuf]) -> Result<bool> {
    for pattern in group {
        if pattern.contains('*') {
            let glob = Glob::new(pattern.as_str())?.compile_matcher();
            let matched = files.iter().any(|f| {
                f.parent() == Some(subdir)
                    && f.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| glob.is_match(n))
            });
            if matched {
                return Ok(true);
            }
        } else {
            let candidate = subdir.join(pattern.as_str());
            if files.contains(&candidate) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn first_level_subdirs(root: &Path, files: &[PathBuf]) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut subdirs = Vec::new();
    for file in files {
        let Ok(rel) = file.strip_prefix(root) else {
            continue;
        };
        let mut components = rel.components();
        let Some(first) = components.next() else {
            continue;
        };
        if components.next().is_none() {
            continue;
        }
        let name = first.as_os_str().to_str().unwrap_or("");
        let subdir = root.join(name);
        if seen.insert(subdir.clone()) {
            subdirs.push(subdir);
        }
    }
    subdirs.sort();
    subdirs
}

#[cfg(test)]
mod tests;
