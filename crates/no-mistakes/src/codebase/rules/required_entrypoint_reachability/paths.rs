use crate::codebase::dependencies::graph::EdgeKind;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn resolve_entrypoint(root: &Path, configured: &str) -> Option<PathBuf> {
    let configured = Path::new(configured.trim_start_matches("./"));
    let path = if configured.is_absolute() {
        configured.to_path_buf()
    } else {
        root.join(configured)
    };
    let path = crate::codebase::ts_resolver::normalize_path(&path);
    path.starts_with(root).then_some(path)
}

pub(super) fn runtime_edge_kinds() -> HashSet<EdgeKind> {
    [
        EdgeKind::Import,
        EdgeKind::DynamicImport,
        EdgeKind::Require,
        EdgeKind::WorkspaceImport,
        EdgeKind::AssetImport,
    ]
    .into_iter()
    .collect()
}
