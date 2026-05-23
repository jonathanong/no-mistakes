mod layout_check;

use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "strict-package-layout";

const DEFAULT_TEST_DIR: &str = "__tests__";
const DEFAULT_TEST_PATTERNS: &[&str] = &["*.test.*", "*.spec.*"];

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct PackageLayoutSpec {
    /// Parent directory that contains individual package subdirectories.
    pub(crate) root: PathBuf,
    pub(crate) source_extension: String,
    pub(crate) allowed_root_files: Vec<String>,
    pub(crate) allowed_subdirs: Vec<String>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) test_file_patterns: Vec<String>,
    pub(crate) test_dir_name: String,
    pub(crate) packages: Vec<PackageLayoutSpec>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let files = discover_files(root, &config.filesystem.skip_directories);
    check_with_files(root, config, &files)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        findings.extend(scan(root, &opts, files));
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

pub(crate) fn build_test_globset(patterns: &[&str]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        if let Ok(glob) = GlobBuilder::new(pattern).literal_separator(false).build() {
            builder.add(glob);
        }
    }
    builder.build().unwrap_or_default()
}

pub(crate) fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Vec<RuleFinding> {
    if opts.packages.is_empty() {
        return Vec::new();
    }
    let owned_patterns: Vec<String>;
    let patterns: Vec<&str> = if opts.test_file_patterns.is_empty() {
        DEFAULT_TEST_PATTERNS.to_vec()
    } else {
        owned_patterns = opts.test_file_patterns.clone();
        owned_patterns.iter().map(String::as_str).collect()
    };
    let test_dir = if opts.test_dir_name.is_empty() {
        DEFAULT_TEST_DIR
    } else {
        &opts.test_dir_name
    };
    let test_globs = build_test_globset(&patterns);

    let mut findings = Vec::new();
    for spec in &opts.packages {
        let spec_root = if spec.root.is_absolute() {
            spec.root.clone()
        } else {
            root.join(&spec.root)
        };
        let pkg_dirs: HashSet<PathBuf> = files
            .iter()
            .filter(|f| f.starts_with(&spec_root))
            .filter_map(|f| f.strip_prefix(&spec_root).ok())
            .filter_map(|rel| rel.components().next())
            .filter_map(|c| c.as_os_str().to_str())
            .map(|d| spec_root.join(d))
            .filter(|d| d.is_dir())
            .collect();
        for pkg_dir in &pkg_dirs {
            for file in files {
                let Some(rel) = file.strip_prefix(pkg_dir).ok() else {
                    continue;
                };
                let full_path = relative_slash_path(root, file);
                if let Some(msg) = check_relative(rel, spec, test_dir, &test_globs, &full_path) {
                    findings.push(RuleFinding {
                        rule: RULE_ID.to_string(),
                        file: full_path,
                        line: 1,
                        message: msg,
                        import: None,
                        target: None,
                    });
                }
            }
        }
    }
    findings
}

pub(crate) fn check_relative(
    rel: &Path,
    spec: &PackageLayoutSpec,
    test_dir: &str,
    test_globs: &GlobSet,
    full_path: &str,
) -> Option<String> {
    let components: Vec<&str> = rel
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    if components.is_empty() {
        return None;
    }
    let file_name = *components.last().unwrap();
    match components.len() {
        1 => layout_check::check_root_file(file_name, spec, test_globs, full_path),
        2 => layout_check::check_one_deep(
            components[0],
            file_name,
            spec,
            test_dir,
            test_globs,
            full_path,
        ),
        3 => layout_check::check_two_deep(
            components[0],
            components[1],
            file_name,
            spec,
            test_dir,
            test_globs,
            full_path,
        ),
        _ => Some(format!(
            "{full_path}: nested subdirectories beyond one level are not allowed"
        )),
    }
}

#[cfg(test)]
mod tests;
