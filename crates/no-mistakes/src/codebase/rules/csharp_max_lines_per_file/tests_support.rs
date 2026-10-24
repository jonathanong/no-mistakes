use super::scan::check_source;
use super::*;

pub(super) fn check_file(
    path: &Path,
    root: &Path,
    limit: usize,
    defer_suppression: bool,
) -> Option<RuleFinding> {
    let sources = super::super::source_store_for_files(&[path.to_path_buf()]);
    let content =
        match super::super::source_access::checked_source(&sources, path, root, super::RULE_ID) {
            Ok(content) => content,
            Err(finding) => return Some(*finding),
        };
    check_source(path, root, &content, limit, defer_suppression)
}
