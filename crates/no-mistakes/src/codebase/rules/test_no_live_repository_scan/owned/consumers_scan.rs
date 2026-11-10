//! Importer scans, once per file for the run.
use super::Consumed;
use crate::codebase::ts_resolver::normalize_path;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(super) struct ScanMemo {
    cache: FxHashMap<PathBuf, Vec<usize>>,
    pub(in super::super) walks: u64,
    pub(in super::super) hits: u64,
}

impl Consumed {
    pub(in super::super) fn targeting(
        &self,
        file: &Path,
    ) -> Vec<&super::super::route_index::RouteHit> {
        let key = normalize_path(file);
        if let Some(indexes) = self.cached_indexes(&key) {
            return self.hits_at(&indexes);
        }
        let indexes = super::super::route_index::targeting_indexes(&self.hits, &self.via, file);
        self.store_indexes(key, &indexes);
        self.hits_at(&indexes)
    }

    fn cached_indexes(&self, key: &Path) -> Option<Vec<usize>> {
        let mut memo = self.scan_memo.borrow_mut();
        let indexes = memo.cache.get(key)?.clone();
        memo.hits += 1;
        Some(indexes)
    }

    fn store_indexes(&self, key: PathBuf, indexes: &[usize]) {
        let mut memo = self.scan_memo.borrow_mut();
        memo.walks += 1;
        memo.cache.insert(key, indexes.to_vec());
    }

    pub(in super::super) fn report_targeting(&self) {
        let memo = self.scan_memo.borrow();
        eprintln!(
            "scan-resolve targeting walks={} hits={}",
            memo.walks, memo.hits
        );
    }

    fn hits_at(&self, indexes: &[usize]) -> Vec<&super::super::route_index::RouteHit> {
        indexes.iter().map(|index| &self.hits[*index]).collect()
    }
}
