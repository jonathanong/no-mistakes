use crate::codebase::ts_resolver::{
    normalize_path, ResolverResultCache, ResolverScopeKey, TsConfig,
};
use crate::codebase::ts_source::{FileInventory, SourceStore, VisiblePathSnapshot};
use crate::diagnostics::InvocationObserver;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod io;
mod parsing;

/// Invocation-owned analysis gateways and memoized work.
///
/// The session is deliberately stateless across invocations. Its caches hold
/// both successes and failures, preventing repeated filesystem work while a
/// CLI or N-API request is active. OXC programs are not stored here because
/// they must remain on the parsing thread; parser callbacks return owned facts.
#[doc(hidden)]
pub struct AnalysisSession {
    observer: Option<Arc<InvocationObserver>>,
    datasets: DashMap<PathBuf, Arc<crate::codebase::analysis_dataset::AnalysisDataset>>,
    supplemental_sources: Arc<SourceStore>,
    resolver_caches: DashMap<ResolverScopeKey, Arc<ResolverResultCache>>,
    parse_attempts: Option<DashMap<PathBuf, u64>>,
}

type SourceReadResult = Result<Arc<str>, SourceReadError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceReadError {
    pub path: PathBuf,
    detail: Arc<str>,
}

impl fmt::Display for SourceReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.detail)
    }
}

impl std::error::Error for SourceReadError {}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SessionWorkSnapshot {
    pub source_reads: BTreeMap<PathBuf, u64>,
    pub parse_attempts: BTreeMap<PathBuf, u64>,
}

impl AnalysisSession {
    pub fn new(observer: Option<Arc<InvocationObserver>>) -> Arc<Self> {
        let collect_keyed_work = observer.as_ref().is_some_and(|observer| observer.verbose());
        let supplemental_sources = Arc::new(SourceStore::new_observed(
            Arc::new(FileInventory::from_paths(&[])),
            observer.clone(),
        ));
        Arc::new(Self {
            observer,
            datasets: DashMap::new(),
            supplemental_sources,
            resolver_caches: DashMap::new(),
            parse_attempts: collect_keyed_work.then(DashMap::new),
        })
    }

    /// Construct a disabled session for stable public wrappers that do not
    /// expose diagnostics. No clocks or work ledgers are created.
    pub fn disabled() -> Arc<Self> {
        Self::new(None)
    }

    pub fn observer(&self) -> Option<&Arc<InvocationObserver>> {
        self.observer.as_ref()
    }

    /// Return the invocation-owned result cache for one exact resolver scope.
    pub(crate) fn resolver_cache(
        &self,
        tsconfig: &TsConfig,
        visible: Option<&std::collections::HashSet<PathBuf>>,
    ) -> Arc<ResolverResultCache> {
        let scope = ResolverScopeKey::new(tsconfig, visible);
        match self.resolver_caches.entry(scope) {
            Entry::Occupied(entry) => Arc::clone(entry.get()),
            Entry::Vacant(entry) => {
                let cache = Arc::new(DashMap::new());
                entry.insert(Arc::clone(&cache));
                cache
            }
        }
    }

    /// Return the canonical visible-path snapshot for `root`, discovering a
    /// normalized root no more than once during this invocation.
    pub fn visible_paths(&self, root: &Path) -> Arc<VisiblePathSnapshot> {
        let root = normalize_path(root);
        self.increment("discovery.requests", 1);
        match self.datasets.entry(root.clone()) {
            Entry::Occupied(entry) => {
                self.increment("discovery.cache_hits", 1);
                entry.get().visible_paths_arc()
            }
            Entry::Vacant(entry) => {
                let dataset = Arc::new(
                    crate::codebase::analysis_dataset::AnalysisDataset::new_observed(
                        &root,
                        self.observer.clone(),
                    ),
                );
                let snapshot = dataset.visible_paths_arc();
                entry.insert(dataset);
                snapshot
            }
        }
    }

    /// Seed a snapshot prepared by an enclosing pipeline. This is used by
    /// compatibility adapters while callers migrate to session discovery.
    pub fn insert_visible_paths(&self, root: &Path, snapshot: Arc<VisiblePathSnapshot>) {
        let root = normalize_path(root);
        self.datasets.entry(root.clone()).or_insert_with(|| {
            Arc::new(
                crate::codebase::analysis_dataset::AnalysisDataset::from_snapshot_observed(
                    &root,
                    snapshot,
                    self.observer.clone(),
                ),
            )
        });
    }

    pub(crate) fn dataset(
        &self,
        root: &Path,
    ) -> Arc<crate::codebase::analysis_dataset::AnalysisDataset> {
        let root = normalize_path(root);
        match self.datasets.entry(root.clone()) {
            Entry::Occupied(entry) => Arc::clone(entry.get()),
            Entry::Vacant(entry) => {
                let dataset = Arc::new(
                    crate::codebase::analysis_dataset::AnalysisDataset::new_observed(
                        &root,
                        self.observer.clone(),
                    ),
                );
                entry.insert(Arc::clone(&dataset));
                dataset
            }
        }
    }

    fn source_store_for_path(&self, path: &Path) -> Arc<SourceStore> {
        let path = normalize_path(path);
        self.datasets
            .iter()
            .filter(|entry| path.starts_with(entry.key()))
            .max_by_key(|entry| entry.key().components().count())
            .map(|entry| entry.value().sources_for(entry.key()))
            .unwrap_or_else(|| Arc::clone(&self.supplemental_sources))
    }
}

#[cfg(test)]
mod tests;
