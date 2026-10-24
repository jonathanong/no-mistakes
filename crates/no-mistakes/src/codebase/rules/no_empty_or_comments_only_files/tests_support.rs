use super::*;

pub(super) fn check_file(path: &Path, root: &Path) -> Vec<RuleFinding> {
    let sources = super::super::source_store_for_files(&[path.to_path_buf()]);
    check_file_with_sources(path, root, &sources)
}
