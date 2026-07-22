use crate::codebase::ts_resolver::{
    normalize_path, ResolverCacheScopeKey, ResolverResultCache, TsConfig,
};
use crate::codebase::ts_source::{FileInventory, SourceStore, VisiblePathSnapshot};
use crate::diagnostics::InvocationObserver;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

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
    datasets: DashMap<PathBuf, Arc<DatasetCell>>,
    supplemental_sources: Arc<SourceStore>,
    resolver_caches: DashMap<ResolverCacheScopeKey, Arc<ResolverResultCache>>,
    parse_attempts: Option<DashMap<PathBuf, u64>>,
}

type AnalysisDataset = crate::codebase::analysis_dataset::AnalysisDataset;
type DatasetCell = OnceLock<Arc<AnalysisDataset>>;
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
        self.resolver_cache_for_scope(ResolverCacheScopeKey::new(tsconfig, visible, None, &[]))
    }

    pub(crate) fn resolver_cache_for_scope(
        &self,
        scope: ResolverCacheScopeKey,
    ) -> Arc<ResolverResultCache> {
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
        self.increment("discovery.requests", 1);
        let (dataset, cache_hit) = self.dataset_with(root, |root| {
            AnalysisDataset::new_observed(root, self.observer.clone())
        });
        if cache_hit {
            self.increment("discovery.cache_hits", 1);
        }
        dataset.visible_paths_arc()
    }

    /// Seed a snapshot prepared by an enclosing pipeline. This is used by
    /// compatibility adapters while callers migrate to session discovery.
    pub fn insert_visible_paths(&self, root: &Path, snapshot: Arc<VisiblePathSnapshot>) {
        let (root, cell, _) = self.dataset_cell(root);
        cell.get_or_init(|| {
            Arc::new(AnalysisDataset::from_snapshot_observed(
                &root,
                snapshot,
                self.observer.clone(),
            ))
        });
    }

    pub(crate) fn dataset(&self, root: &Path) -> Arc<AnalysisDataset> {
        self.dataset_with(root, |root| {
            AnalysisDataset::new_observed(root, self.observer.clone())
        })
        .0
    }

    fn dataset_with(
        &self,
        root: &Path,
        initialize: impl FnOnce(&Path) -> AnalysisDataset,
    ) -> (Arc<AnalysisDataset>, bool) {
        let (root, cell, cache_hit) = self.dataset_cell(root);

        let dataset = cell.get_or_init(|| Arc::new(initialize(&root)));
        (Arc::clone(dataset), cache_hit)
    }

    fn dataset_cell(&self, root: &Path) -> (PathBuf, Arc<DatasetCell>, bool) {
        let root = normalize_path(root);
        let (cell, cache_hit) = match self.datasets.entry(root.clone()) {
            Entry::Occupied(entry) => (Arc::clone(entry.get()), true),
            Entry::Vacant(entry) => {
                let cell = Arc::new(OnceLock::new());
                entry.insert(Arc::clone(&cell));
                (cell, false)
            }
        };

        // Callers initialize only after this entry match drops the DashMap
        // shard guard, keeping discovery outside the registry lock.
        (root, cell, cache_hit)
    }

    fn dataset_from_cell(&self, root: &Path, cell: &DatasetCell) -> Arc<AnalysisDataset> {
        Arc::clone(
            cell.get_or_init(|| {
                Arc::new(AnalysisDataset::new_observed(root, self.observer.clone()))
            }),
        )
    }

    fn source_store_for_path(&self, path: &Path) -> Arc<SourceStore> {
        let path = normalize_path(path);
        let matching_dataset = self
            .datasets
            .iter()
            .filter(|entry| path.starts_with(entry.key()))
            .max_by_key(|entry| entry.key().components().count())
            .map(|entry| (entry.key().clone(), Arc::clone(entry.value())));

        matching_dataset
            .map(|(root, cell)| self.dataset_from_cell(&root, &cell).sources_for(&root))
            .unwrap_or_else(|| Arc::clone(&self.supplemental_sources))
    }
}

#[cfg(test)]
mod tests;
