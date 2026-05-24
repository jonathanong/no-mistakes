use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::Glob;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "require-test-per-subdir";

const DEFAULT_TEST_GLOB: &str = "*.test.*";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) roots: Vec<PathBuf>,
    pub(crate) exclude_dirs: Vec<String>,
    pub(crate) test_glob: Option<String>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let skip = &config.filesystem.skip_directories;
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        if opts.roots.is_empty() {
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
        if opts.roots.is_empty() {
            continue;
        }
        findings.extend(scan(root, &opts, files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let test_glob_str = opts.test_glob.as_deref().unwrap_or(DEFAULT_TEST_GLOB);
    let glob = Glob::new(test_glob_str)?.compile_matcher();
    let exclude_set: HashSet<&str> = opts.exclude_dirs.iter().map(String::as_str).collect();
    let mut findings = Vec::new();

    for rule_root in &opts.roots {
        let abs_root = if rule_root.is_absolute() {
            rule_root.clone()
        } else {
            root.join(rule_root)
        };

        let subdirs = first_level_subdirs(&abs_root, files, &exclude_set);
        for subdir in subdirs {
            let has_test = files.iter().any(|f| {
                f.starts_with(&subdir)
                    && f.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| glob.is_match(n))
            });
            if !has_test {
                let rel = relative_slash_path(root, &subdir);
                findings.push(RuleFinding {
                    rule: RULE_ID.to_string(),
                    file: rel.clone(),
                    line: 1,
                    message: format!("{rel}: no test file matching '{}' found", test_glob_str),
                    import: None,
                    target: None,
                });
            }
        }
    }
    Ok(findings)
}

fn first_level_subdirs(
    root: &Path,
    files: &[PathBuf],
    exclude_set: &HashSet<&str>,
) -> Vec<PathBuf> {
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
        // must have at least one more component (i.e. this is not a root-level file)
        if components.next().is_none() {
            continue;
        }
        let name = first.as_os_str().to_str().unwrap_or("");
        if exclude_set.contains(name) {
            continue;
        }
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
