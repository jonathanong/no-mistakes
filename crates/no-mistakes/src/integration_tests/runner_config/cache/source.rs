use super::REQUEST_CACHES;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

fn cached_sources() -> Option<Arc<crate::codebase::ts_source::SourceStore>> {
    REQUEST_CACHES.with(|caches| {
        caches
            .borrow()
            .last()
            .and_then(|request| request.sources.clone())
    })
}

fn read_from_source_store(
    sources: &crate::codebase::ts_source::SourceStore,
    path: &Path,
) -> Result<Arc<str>> {
    sources
        .read_path(path)
        .map_err(|error| anyhow::anyhow!("reading {}: {}", path.display(), error))
}

pub(in crate::integration_tests) fn read_request_source(path: &Path) -> Result<Arc<str>> {
    match cached_sources() {
        Some(sources) => read_from_source_store(&sources, path),
        None => std::fs::read_to_string(path)
            .map(Arc::<str>::from)
            .map_err(anyhow::Error::from),
    }
}

pub(in crate::integration_tests::runner_config) fn read_request_source_with_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    path: &Path,
) -> Result<Arc<str>> {
    match cached_sources() {
        Some(sources) => read_from_source_store(&sources, path),
        None => session
            .read_source(path)
            .map_err(|error| anyhow::anyhow!("reading {}: {}", path.display(), error)),
    }
}

#[cfg(test)]
mod tests;
