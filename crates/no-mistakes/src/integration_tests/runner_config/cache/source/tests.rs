use super::*;

fn with_cached_sources<T>(
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
    collect: impl FnOnce() -> T,
) -> T {
    super::super::REQUEST_CACHES.with(|caches| {
        caches
            .borrow_mut()
            .push(super::super::RequestCache::new(None, Some(sources)));
    });
    let result = collect();
    super::super::REQUEST_CACHES.with(|caches| {
        caches.borrow_mut().pop();
    });
    result
}

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

#[test]
fn read_request_source_uses_the_cached_source_store() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cfg.ts");
    std::fs::write(&path, "export const ready = 1").unwrap();
    let inventory =
        crate::codebase::ts_source::FileInventory::from_paths(std::slice::from_ref(&path));
    let sources = std::sync::Arc::new(crate::codebase::ts_source::SourceStore::new(
        std::sync::Arc::new(inventory),
    ));
    let source = with_cached_sources(sources, || read_request_source(&path)).unwrap();
    assert!(source.contains("ready"));
}

#[test]
fn read_request_source_reports_cached_store_failures() {
    let missing = std::path::PathBuf::from("/definitely-missing-runner-config.ts");
    let inventory =
        crate::codebase::ts_source::FileInventory::from_paths(std::slice::from_ref(&missing));
    let sources = std::sync::Arc::new(crate::codebase::ts_source::SourceStore::new(
        std::sync::Arc::new(inventory),
    ));
    let error = with_cached_sources(sources.clone(), || read_request_source(&missing)).unwrap_err();
    assert!(error.to_string().contains("reading"));
    let session = crate::codebase::analysis_session::AnalysisSession::new(None);
    let error = with_cached_sources(sources, || {
        read_request_source_with_session(&session, &missing)
    })
    .unwrap_err();
    assert!(error.to_string().contains("reading"));
}
