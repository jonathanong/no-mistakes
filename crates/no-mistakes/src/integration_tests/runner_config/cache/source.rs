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

pub(in crate::integration_tests) fn read_request_source(path: &Path) -> Result<Arc<str>> {
    match cached_sources() {
        Some(sources) => sources
            .read_path(path)
            .map_err(|error| anyhow::anyhow!("reading {}: {}", path.display(), error)),
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
        Some(sources) => sources
            .read_path(path)
            .map_err(|error| anyhow::anyhow!("reading {}: {}", path.display(), error)),
        None => session
            .read_source(path)
            .map_err(|error| anyhow::anyhow!("reading {}: {}", path.display(), error)),
    }
}
