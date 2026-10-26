use super::*;
use crate::codebase::ts_source::has_disable_file_comment;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) excludes: Vec<String>,
    pub(crate) roots: Option<Vec<PathBuf>>,
}

pub(super) fn normalize_roots(
    opts: &Options,
    root: &Path,
    target_roots: &[PathBuf],
) -> Vec<PathBuf> {
    super::super::rust_rules_combined::tests_support::normalize_option_roots(
        root,
        target_roots,
        &opts.roots,
    )
}

pub(super) fn is_excluded(root: &Path, path: &Path, excludes: &[String]) -> bool {
    super::super::rust_rules_combined::tests_support::excluded_path(root, path, excludes)
}

pub(super) fn scan(root: &Path, _opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file(path, root))
        .collect();
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    Ok(findings)
}

pub(crate) fn check_file(path: &Path, root: &Path) -> Vec<RuleFinding> {
    let sources = super::super::source_store_for_files(&[path.to_path_buf()]);
    let content = match super::super::source_access::checked_source(&sources, path, root, RULE_ID) {
        Ok(content) => content,
        Err(finding) => return vec![*finding],
    };
    if has_disable_file_comment(&content, RULE_ID) {
        return Vec::new();
    }
    // Optimization: avoid expensive AST parsing if the file doesn't contain cfg and test
    if !content.contains("cfg") || !content.contains("test") {
        return Vec::new();
    }
    let parsed = match syn::parse_file(&content) {
        Ok(parsed) => parsed,
        Err(error) => {
            return vec![super::super::source_access::unanalyzable_source(
                root,
                path,
                RULE_ID,
                format!("cannot parse Rust: {error}"),
            )]
        }
    };
    findings_from_parsed(path, root, &parsed)
}
