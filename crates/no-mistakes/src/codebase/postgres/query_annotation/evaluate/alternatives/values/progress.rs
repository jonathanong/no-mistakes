//! Exact arm-local work counters; publish outside the per-binding hot path.
use super::Value;
use crate::diagnostics::InvocationObserver;
use std::sync::Arc;

const INTERVAL: u64 = 1 << 20;

pub(in super::super) struct Walk {
    observer: Arc<InvocationObserver>,
    progress: bool,
    nodes: u64,
    containers: u64,
    comparisons: u64,
    completed: u64,
    after_empty_skips: u64,
    unchanged_bindings: u64,
    memo_hits: u64,
    memo_misses: u64,
    published: [u64; 8],
}

impl Walk {
    pub(in super::super) fn new() -> Option<Self> {
        let observer = crate::diagnostics::current().filter(|observer| observer.verbose())?;
        Some(Self {
            progress: observer.progress_enabled(),
            observer,
            nodes: 0,
            containers: 0,
            comparisons: 0,
            completed: 0,
            after_empty_skips: 0,
            unchanged_bindings: 0,
            memo_hits: 0,
            memo_misses: 0,
            published: [0; 8],
        })
    }

    pub(in super::super) fn memoized(&mut self, hit: bool) {
        self.memo_hits += u64::from(hit);
        self.memo_misses += u64::from(!hit);
    }

    pub(in super::super) fn unchanged_binding(&mut self) {
        self.unchanged_bindings += 1;
    }

    pub(super) fn after_empty_skip(&mut self) {
        self.after_empty_skips += 1;
    }

    pub(super) fn comparison(&mut self) {
        self.comparisons += 1;
    }

    pub(super) fn visit(&mut self, value: &Value) {
        self.nodes += 1;
        if matches!(
            value,
            Value::Aggregate(_)
                | Value::References(_)
                | Value::Object(_)
                | Value::Record(_, _)
                | Value::Joined(_)
                | Value::Possible(_)
        ) {
            self.containers += 1;
        }
        if self.progress && self.nodes.is_multiple_of(INTERVAL) {
            self.flush();
            self.emit();
        }
    }

    pub(super) fn complete(&mut self) {
        self.completed += 1;
    }

    fn flush(&mut self) {
        let totals = [
            self.comparisons,
            self.completed,
            self.nodes,
            self.containers,
            self.after_empty_skips,
            self.unchanged_bindings,
            self.memo_hits,
            self.memo_misses,
        ];
        if totals == self.published {
            return;
        }
        let names = [
            "postgres.annotation.prefix_walks",
            "postgres.annotation.prefix_walks_completed",
            "postgres.annotation.prefix_nodes",
            "postgres.annotation.prefix_containers",
            "postgres.annotation.prefix_after_empty_skips",
            "postgres.annotation.prefix_unchanged_binding_skips",
            "postgres.annotation.prefix_comparison_memo_hits",
            "postgres.annotation.prefix_comparison_memo_misses",
        ];
        for ((name, value), previous) in names.into_iter().zip(totals).zip(self.published) {
            self.observer.increment(name, value - previous);
        }
        self.observer.increment(
            "postgres.annotation.prefix_compare",
            self.comparisons - self.published[0],
        );
        self.observer
            .increment("postgres.annotation.prefix_observer_batches", 1);
        self.published = totals;
    }

    fn emit(&self) {
        // Bounded best-effort delivery; no per-node uniqueness map or span.
        self.observer.emit_progress_output(format!(
            "[work] postgres.annotation.prefix_nodes_current_arm: {}\n\
             [work] postgres.annotation.prefix_containers_current_arm: {}\n",
            self.nodes, self.containers,
        ));
    }
}

impl Drop for Walk {
    fn drop(&mut self) {
        self.flush();
        if self.progress && self.nodes >= INTERVAL && !self.nodes.is_multiple_of(INTERVAL) {
            self.emit();
        }
    }
}

#[cfg(test)]
mod tests;
