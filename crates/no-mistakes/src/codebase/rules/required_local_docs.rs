mod doc_section;
mod scan_pkg;

pub(crate) use doc_section::check_required_doc_section_with_files;
pub use doc_section::{check_required_doc_section, DocSectionOptions};

use super::RuleFinding;
use crate::codebase::ts_source::discover_files;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use serde::Deserialize;
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

fn make_scan_config(opts: &Options) -> (String, Vec<String>, Vec<String>, GlobSet) {
    let req = if opts.required_file.is_empty() {
        DEFAULT_REQUIRED_FILE.to_string()
    } else {
        opts.required_file.clone()
    };
    let ext = if opts.code_extensions.is_empty() {
        DEFAULT_CODE_EXTENSIONS
            .iter()
            .map(|&s| s.to_string())
            .collect()
    } else {
        opts.code_extensions.clone()
    };
    let excl: Vec<String> = if opts.test_exclude_patterns.is_empty() {
        DEFAULT_TEST_EXCLUDE.iter().map(|s| s.to_string()).collect()
    } else {
        opts.test_exclude_patterns.clone()
    };
    let globs = build_exclude_globs(&excl.iter().map(String::as_str).collect::<Vec<_>>());
    (req, ext, excl, globs)
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

pub(crate) fn scan<'a>(root: &Path, opts: &Options, files: &'a [PathBuf]) -> Vec<RuleFinding> {
    if opts.roots.is_empty() {
        return Vec::new();
    }
    let (req_file, ext_s, excl_s, globs) = make_scan_config(opts);
    let ext: Vec<&str> = ext_s.iter().map(String::as_str).collect();
    let excl: Vec<&str> = excl_s.iter().map(String::as_str).collect();
    let ctx = scan_pkg::ScanCtx {
        req_file,
        ext,
        excl,
        globs,
        file_set: files.iter().collect(),
        files,
    };
    opts.roots
        .par_iter()
        .flat_map(|root_rel| {
            let pkg_root = if root_rel.is_absolute() {
                root_rel.clone()
            } else {
                root.join(root_rel)
            };
            scan_pkg::scan_pkg(root, &pkg_root, root_rel, &ctx)
        })
        .collect()
}

#[cfg(test)]
mod tests;
