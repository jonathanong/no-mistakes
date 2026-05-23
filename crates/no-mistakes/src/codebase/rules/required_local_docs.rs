mod doc_section;

pub(crate) use doc_section::check_required_doc_section_with_files;
pub use doc_section::{check_required_doc_section, DocSectionOptions};

use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "required-local-docs";
pub const REQUIRED_DOC_SECTION_RULE_ID: &str = "required-doc-section";

pub(crate) const DEFAULT_CODE_EXTENSIONS: &[&str] =
    &["ts", "mts", "cts", "js", "jsx", "tsx", "sql", "rs"];
pub(crate) const DEFAULT_TEST_EXCLUDE: &[&str] = &["*.test.*", "*.spec.*", "__tests__"];
const DEFAULT_REQUIRED_FILE: &str = "README.md";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) roots: Vec<PathBuf>,
    pub(crate) required_file: String,
    pub(crate) code_extensions: Vec<String>,
    pub(crate) test_exclude_patterns: Vec<String>,
}

struct ScanConfig {
    required_file: String,
    ext_strs: Vec<String>,
    excl_strs: Vec<String>,
    excl_globs: GlobSet,
}

impl ScanConfig {
    fn new(opts: &Options) -> Self {
        let required_file = if opts.required_file.is_empty() {
            DEFAULT_REQUIRED_FILE.to_string()
        } else {
            opts.required_file.clone()
        };
        let ext_strs: Vec<String> = if opts.code_extensions.is_empty() {
            DEFAULT_CODE_EXTENSIONS
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            opts.code_extensions.clone()
        };
        let excl_strs: Vec<String> = if opts.test_exclude_patterns.is_empty() {
            DEFAULT_TEST_EXCLUDE.iter().map(|s| s.to_string()).collect()
        } else {
            opts.test_exclude_patterns.clone()
        };
        let excl_refs: Vec<&str> = excl_strs.iter().map(String::as_str).collect();
        let excl_globs = build_exclude_globs(&excl_refs);
        Self {
            required_file,
            ext_strs,
            excl_strs,
            excl_globs,
        }
    }
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

pub(crate) fn build_exclude_globs(patterns: &[&str]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for p in patterns {
        if let Ok(glob) = GlobBuilder::new(p).literal_separator(false).build() {
            builder.add(glob);
        }
    }
    builder.build().unwrap_or_default()
}

pub(crate) fn is_code_file(
    file: &Path,
    extensions: &[&str],
    excl_strs: &[&str],
    excl_globs: &GlobSet,
) -> bool {
    let ext_ok = file
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| extensions.contains(&e));
    if !ext_ok {
        return false;
    }
    let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if excl_globs.is_match(name) {
        return false;
    }
    let literals: Vec<&str> = excl_strs
        .iter()
        .copied()
        .filter(|p| !p.contains('*') && !p.contains('?'))
        .collect();
    literals.is_empty()
        || !file.components().any(|c| {
            c.as_os_str()
                .to_str()
                .is_some_and(|s| literals.contains(&s))
        })
}

pub(crate) fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Vec<RuleFinding> {
    if opts.roots.is_empty() {
        return Vec::new();
    }
    let cfg = ScanConfig::new(opts);
    let file_set: HashSet<&PathBuf> = files.iter().collect();
    let mut findings = Vec::new();
    for root_rel in &opts.roots {
        let pkg_root = if root_rel.is_absolute() {
            root_rel.clone()
        } else {
            root.join(root_rel)
        };
        findings.extend(scan_root(root, &pkg_root, root_rel, &cfg, &file_set, files));
    }
    findings
}

fn scan_root<'a>(
    root: &Path,
    pkg_root: &Path,
    pkg_root_rel: &Path,
    cfg: &ScanConfig,
    file_set: &HashSet<&'a PathBuf>,
    files: &'a [PathBuf],
) -> Vec<RuleFinding> {
    let ext_strs: Vec<&str> = cfg.ext_strs.iter().map(String::as_str).collect();
    let excl_strs: Vec<&str> = cfg.excl_strs.iter().map(String::as_str).collect();
    let mut subdirs_with_code: HashSet<String> = HashSet::new();
    for file in files {
        if !file.starts_with(pkg_root) {
            continue;
        }
        let rel = match file.strip_prefix(pkg_root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let comps: Vec<&str> = rel
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();
        if comps.len() >= 2 && is_code_file(file, &ext_strs, &excl_strs, &cfg.excl_globs) {
            subdirs_with_code.insert(comps[0].to_string());
        }
    }
    let root_rel_str = pkg_root_rel.to_string_lossy().replace('\\', "/");
    subdirs_with_code
        .into_iter()
        .filter_map(|subdir| {
            let doc_path = pkg_root.join(&subdir).join(&cfg.required_file);
            if file_set.contains(&doc_path) {
                return None;
            }
            let dir_rel = relative_slash_path(root, &pkg_root.join(&subdir));
            Some(RuleFinding {
                rule: RULE_ID.to_string(),
                file: format!("{root_rel_str}/{subdir}"),
                line: 1,
                message: format!(
                    "{dir_rel}: code-owning directory is missing {}",
                    cfg.required_file
                ),
                import: None,
                target: None,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
