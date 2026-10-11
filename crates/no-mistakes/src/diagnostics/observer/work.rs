//! Request-owned counters: built-in metrics need no map lock on increment.
use dashmap::DashMap;
use rustc_hash::FxBuildHasher;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::OnceLock;

#[derive(Default)]
struct Counter {
    value: AtomicU64,
    present: AtomicBool,
}

pub(super) struct WorkCounters {
    core: Box<[Counter]>,
    extensions: OnceLock<DashMap<&'static str, AtomicU64, FxBuildHasher>>,
}

impl WorkCounters {
    pub(super) fn new() -> Self {
        Self {
            core: (0..NAMES.len()).map(|_| Counter::default()).collect(),
            extensions: OnceLock::new(),
        }
    }

    pub(super) fn increment(&self, metric: &'static str, amount: u64) {
        if let Some(index) = core_index(metric) {
            let counter = &self.core[index];
            counter.value.fetch_add(amount, Ordering::Relaxed);
            // Presence preserves explicitly recorded zeros. Avoid writing this
            // cache line again once the metric has appeared.
            if !counter.present.load(Ordering::Relaxed) {
                counter.present.store(true, Ordering::Relaxed);
            }
            return;
        }
        let extensions = self
            .extensions
            .get_or_init(|| DashMap::with_hasher(FxBuildHasher));
        if let Some(counter) = extensions.get(metric) {
            counter.fetch_add(amount, Ordering::Relaxed);
            return;
        }
        extensions
            .entry(metric)
            .or_default()
            .fetch_add(amount, Ordering::Relaxed);
    }

    pub(super) fn snapshot(&self) -> BTreeMap<&'static str, u64> {
        // Invocation snapshots are finalized after workers join; atomics need
        // no ordering beyond the workers' existing completion synchronization.
        let mut snapshot = BTreeMap::new();
        for (name, counter) in NAMES.iter().zip(self.core.iter()) {
            if counter.present.load(Ordering::Relaxed) {
                snapshot.insert(*name, counter.value.load(Ordering::Relaxed));
            }
        }
        if let Some(extensions) = self.extensions.get() {
            snapshot.extend(
                extensions
                    .iter()
                    .map(|entry| (*entry.key(), entry.value().load(Ordering::Relaxed))),
            );
        }
        snapshot
    }
}

const NAMES: &[&str] = &[
    "analysis.requests",
    "ast.walks",
    "discovery.cache_hits",
    "discovery.candidates",
    "discovery.projections",
    "discovery.requests",
    "discovery.roots",
    "graph.builds",
    "graph.candidate_excluded",
    "graph.candidate_files",
    "graph.edges",
    "graph.nodes",
    "graph.reuses",
    "manifest.cache_hits",
    "manifest.errors",
    "manifest.parses",
    "manifest.requests",
    "output.errors",
    "output.renders",
    "parse.errors",
    "parse.files",
    "parse.files_after_extract",
    "parse.requests",
    "query_diagnostics.builds",
    "resolver.cache_hits",
    "resolver.classification_requests",
    "resolver.classifications",
    "resolver.computations",
    "resolver.requests",
    "resolver.resolved",
    "resolver.scope_cache_requests",
    "resolver.unique_keys",
    "resolver.unresolved",
    "source.bytes",
    "source.cache_hits",
    "source.read_errors",
    "source.reads",
    "source.requests",
    "symbol_index.builds",
    "symbol_index.reuses",
    "test_filter.builds",
    "test_filter.project_filters",
    "traversal.computations",
    "traversal.lazy_nodes",
    "traversal.nodes",
    "traversal.requests",
    "traversal.reuses",
    "ts_facts.collections",
    "ts_facts.files",
    "workspace.builds",
];

fn core_index(metric: &str) -> Option<usize> {
    Some(match metric {
        "analysis.requests" => 0,
        "ast.walks" => 1,
        "discovery.cache_hits" => 2,
        "discovery.candidates" => 3,
        "discovery.projections" => 4,
        "discovery.requests" => 5,
        "discovery.roots" => 6,
        "graph.builds" => 7,
        "graph.candidate_excluded" => 8,
        "graph.candidate_files" => 9,
        "graph.edges" => 10,
        "graph.nodes" => 11,
        "graph.reuses" => 12,
        "manifest.cache_hits" => 13,
        "manifest.errors" => 14,
        "manifest.parses" => 15,
        "manifest.requests" => 16,
        "output.errors" => 17,
        "output.renders" => 18,
        "parse.errors" => 19,
        "parse.files" => 20,
        "parse.files_after_extract" => 21,
        "parse.requests" => 22,
        "query_diagnostics.builds" => 23,
        "resolver.cache_hits" => 24,
        "resolver.classification_requests" => 25,
        "resolver.classifications" => 26,
        "resolver.computations" => 27,
        "resolver.requests" => 28,
        "resolver.resolved" => 29,
        "resolver.scope_cache_requests" => 30,
        "resolver.unique_keys" => 31,
        "resolver.unresolved" => 32,
        "source.bytes" => 33,
        "source.cache_hits" => 34,
        "source.read_errors" => 35,
        "source.reads" => 36,
        "source.requests" => 37,
        "symbol_index.builds" => 38,
        "symbol_index.reuses" => 39,
        "test_filter.builds" => 40,
        "test_filter.project_filters" => 41,
        "traversal.computations" => 42,
        "traversal.lazy_nodes" => 43,
        "traversal.nodes" => 44,
        "traversal.requests" => 45,
        "traversal.reuses" => 46,
        "ts_facts.collections" => 47,
        "ts_facts.files" => 48,
        "workspace.builds" => 49,
        _ => return None,
    })
}

#[cfg(test)]
#[path = "work_tests.rs"]
mod tests;
