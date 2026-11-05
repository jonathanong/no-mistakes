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

pub(super) fn require_sources<'a>(
    graph: &crate::codebase::dependencies::graph::DepGraph,
    files: impl IntoIterator<Item = &'a PathBuf>,
) -> anyhow::Result<()> {
    let files: std::collections::BTreeSet<_> = files.into_iter().collect();
    for file in files {
        if let Some(error) = graph.parse_error(file) {
            return Err(crate::codebase::check_outcome::CheckInputFailure::source(
                file.clone(),
                format!(
                    "{}: required source {} could not be analyzed: {error}",
                    super::RULE_ID,
                    file.display()
                ),
            )
            .into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
