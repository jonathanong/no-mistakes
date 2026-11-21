use crate::codebase::dependencies::graph::{DepGraph, NodeId};
use anyhow::Result;

pub(crate) fn require_import_sources(
    consumer: &str,
    graph: &DepGraph,
    roots: &[NodeId],
    max_depth: Option<usize>,
) -> Result<()> {
    if graph.parse_errors().next().is_none() {
        return Ok(());
    }
    let required = graph.runtime_import_sources_with_root_mocks(roots, max_depth);
    for file in required {
        if let Some(error) = graph.parse_error(&file) {
            return Err(crate::codebase::check_outcome::CheckInputFailure::source(
                file.clone(),
                format!(
                    "{consumer}: required source {} could not be analyzed: {error}",
                    file.display()
                ),
            )
            .into());
        }
    }
    Ok(())
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
