use super::*;

pub(crate) fn normalize_option_roots(
    root: &Path,
    target_roots: &[PathBuf],
    roots: &Option<Vec<PathBuf>>,
) -> Vec<PathBuf> {
    selection::normalize_roots(root, target_roots, roots)
}

pub(crate) fn excluded_path(root: &Path, path: &Path, excludes: &[String]) -> bool {
    selection::is_excluded(root, path, excludes)
}

pub(super) fn scan_file_with_deferred_suppression(
    root: &Path,
    path: &Path,
    work: &RustWork,
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    super::scan::scan_file_outputs(
        root,
        path,
        work,
        sources,
        defer_suppression,
        syn::parse_file,
    )
    .into_iter()
    .map(|(_, finding)| finding)
    .collect()
}

pub(super) fn scan_file_with_source_and_deferred_suppression(
    root: &Path,
    path: &Path,
    work: &RustWork,
    content: &str,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    super::scan::scan_outputs_with_parser(
        root,
        path,
        work,
        content,
        defer_suppression,
        syn::parse_file,
    )
    .into_iter()
    .map(|(_, finding)| finding)
    .collect()
}
