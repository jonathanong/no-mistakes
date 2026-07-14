use super::*;

pub(super) fn resolve_entrypoints_with_files(
    raw_entrypoints: &[PathBuf],
    symbol_entrypoints: &[Option<String>],
    structured_entrypoints: &[bool],
    root: &Path,
    cwd: &Path,
    graph_files: &graph::GraphFiles,
    include_symbols: bool,
) -> Vec<Entrypoint> {
    let workspace = crate::codebase::workspaces::load_indexed_from_files(root, graph_files.all())
        .unwrap_or_default();
    resolve_entrypoints_with_files_and_workspace(EntrypointResolution {
        raw_entrypoints,
        symbol_entrypoints,
        structured_entrypoints,
        root,
        cwd,
        graph_files,
        include_symbols,
        workspace: &workspace,
    })
}
