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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_request_source_without_cache_reads_and_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cfg.ts");
        std::fs::write(&path, "export {}").unwrap();
        assert!(read_request_source(&path).unwrap().contains("export"));
        assert!(read_request_source(&dir.path().join("missing.ts")).is_err());
    }

    #[test]
    fn read_request_source_with_session_surfaces_read_failures() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.ts");
        let session = crate::codebase::analysis_session::AnalysisSession::new(None);
        let error = read_request_source_with_session(&session, &missing).unwrap_err();
        assert!(error.to_string().contains("reading"));
    }
}
