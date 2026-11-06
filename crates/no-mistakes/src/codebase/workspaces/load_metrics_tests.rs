use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::LazyLock;

static LOADS: LazyLock<DashMap<PathBuf, AtomicUsize>> = LazyLock::new(DashMap::new);

macro_rules! workspace_load_metrics {
    ($root:expr) => {
        $crate::codebase::workspaces::load_metrics_tests::record($root);
    };
}

pub(crate) fn record(root: &Path) {
    if let Some(count) = LOADS.get(&super::normalize_path(root)) {
        count.fetch_add(1, Ordering::Relaxed);
    }
}

pub(crate) fn count<T>(root: &Path, run: impl FnOnce() -> T) -> (T, usize) {
    let root = super::normalize_path(root);
    assert!(LOADS.insert(root.clone(), AtomicUsize::new(0)).is_none());
    let result = run();
    (result, LOADS.remove(&root).unwrap().1.into_inner())
}
