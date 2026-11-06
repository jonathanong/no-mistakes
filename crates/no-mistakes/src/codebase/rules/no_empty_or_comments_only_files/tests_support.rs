use super::*;

pub(super) fn check_file(path: &Path, root: &Path) -> Vec<RuleFinding> {
    let sources = super::super::source_store_for_files(&[path.to_path_buf()]);
    let content = match super::super::source_access::checked_source(&sources, path, root, RULE_ID) {
        Ok(content) => content,
        Err(finding) => return vec![*finding],
    };
    check_source(path, root, &content)
}
