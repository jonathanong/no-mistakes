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
    published: [u64; 4],
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
            published: [0; 4],
        })
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
        ];
        if totals == self.published {
            return;
        }
        let names = [
            "postgres.annotation.prefix_walks",
            "postgres.annotation.prefix_walks_completed",
            "postgres.annotation.prefix_nodes",
            "postgres.annotation.prefix_containers",
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
