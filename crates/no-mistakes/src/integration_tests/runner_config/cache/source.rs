use super::REQUEST_CACHES;
use anyhow::Result;
use std::cell::RefCell;
use std::path::Path;
use std::sync::Arc;

thread_local! {
    static SOURCE_SCOPES: RefCell<Vec<Arc<crate::codebase::ts_source::SourceStore>>> = const { RefCell::new(Vec::new()) };
}
struct SourceScope;
impl Drop for SourceScope {
    fn drop(&mut self) {
        SOURCE_SCOPES.with(|sources| {
            sources.borrow_mut().pop();
        });
    }
}

pub(crate) fn with_request_sources<T>(
    sources: Arc<crate::codebase::ts_source::SourceStore>,
    collect: impl FnOnce() -> T,
) -> T {
    SOURCE_SCOPES.with(|scopes| scopes.borrow_mut().push(sources));
    let _scope = SourceScope;
    collect()
}

fn cached_sources() -> Option<Arc<crate::codebase::ts_source::SourceStore>> {
    SOURCE_SCOPES
        .with(|sources| sources.borrow().last().cloned())
        .or_else(|| {
            REQUEST_CACHES.with(|caches| {
                caches
                    .borrow()
                    .last()
                    .and_then(|request| request.sources.clone())
            })
        })
}

fn read_from_source_store(
    sources: &crate::codebase::ts_source::SourceStore,
    path: &Path,
) -> Result<Arc<str>> {
    let read = if crate::codebase::dependencies::extract::is_indexable(path) {
        sources.read_source_path(path)
    } else {
        sources.read_path(path)
    };
    read.map_err(|error| anyhow::anyhow!("reading {}: {}", path.display(), error))
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
