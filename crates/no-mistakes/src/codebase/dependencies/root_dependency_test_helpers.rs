use super::*;

pub(super) fn root_dependency_names(
    root: &Path,
    visible_files: &[PathBuf],
) -> std::collections::HashSet<String> {
    crate::codebase::workspaces::load_indexed_from_files(root, visible_files)
        .map(|workspace| workspace.root_dependency_names().clone())
        .unwrap_or_default()
}
