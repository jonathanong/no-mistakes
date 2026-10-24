use super::*;
use rayon::prelude::*;
use std::path::PathBuf;

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

pub(super) fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let src_max = opts.src_max.unwrap_or(DEFAULT_SRC_MAX);
    let test_max = opts.test_max.unwrap_or(DEFAULT_TEST_MAX);
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| {
            let limit = if is_test_file(root, path) {
                test_max
            } else {
                src_max
            };
            check_file(path, root, limit)
        })
        .collect();
    findings.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(findings)
}

pub(super) fn check_file(path: &Path, root: &Path, limit: usize) -> Option<RuleFinding> {
    let sources = super::super::source_store_for_files(&[path.to_path_buf()]);
    let content = match super::super::source_access::checked_source(&sources, path, root, RULE_ID) {
        Ok(content) => content,
        Err(finding) => return Some(*finding),
    };
    check_source(path, root, &content, limit)
}

pub(super) fn check_source(
    path: &Path,
    root: &Path,
    content: &str,
    limit: usize,
) -> Option<RuleFinding> {
    check_source_with_deferred_suppression(path, root, content, limit, false)
}
