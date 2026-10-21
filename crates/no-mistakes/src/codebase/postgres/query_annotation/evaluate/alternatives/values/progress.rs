//! Observe the existing walk without changing traversal, arena or overwrite order.
use super::Value;
use crate::diagnostics::InvocationObserver;
use crate::fx::FxHashSet;
use std::sync::Arc;

const INTERVAL: u64 = 1 << 20;

pub(super) struct Walk {
    observer: Arc<InvocationObserver>,
    nodes: u64,
    containers: u64,
    unique: FxHashSet<usize>,
}

impl Walk {
    pub(super) fn new() -> Option<Self> {
        let observer = crate::diagnostics::current().filter(|observer| observer.verbose())?;
        observer.increment("postgres.annotation.prefix_walks", 1);
        observer.increment("postgres.annotation.prefix_walks_completed", 0);
        Some(Self {
            observer,
            nodes: 0,
            containers: 0,
            unique: FxHashSet::default(),
        })
    }

    pub(super) fn visit(&mut self, value: &Value) {
        self.nodes += 1;
        match value {
            Value::Aggregate(values)
            | Value::References(values)
            | Value::Object(values)
            | Value::Record(values, _)
            | Value::Joined(values)
            | Value::Possible(values) => {
                self.containers += 1;
                self.unique.insert(values.identity() as usize);
            }
            _ => {}
        }
        if self.nodes.is_multiple_of(INTERVAL) {
            self.observer
                .increment("postgres.annotation.prefix_nodes", INTERVAL);
            self.emit();
        }
    }

    pub(super) fn complete(&self) {
        self.observer
            .increment("postgres.annotation.prefix_walks_completed", 1);
    }

    fn emit(&self) {
        if self.observer.progress_enabled() {
            // The existing bounded writer may drop diagnostics; never fall back
            // to a blocking stderr write. These counts are local to this walk.
            self.observer.emit_progress_output(format!(
                "[work] postgres.annotation.prefix_nodes_current_walk: {}\n\
                 [work] postgres.annotation.prefix_unique_containers_current_walk: {}\n",
                self.nodes,
                self.unique.len(),
            ));
        }
    }
}

impl Drop for Walk {
    fn drop(&mut self) {
        self.observer
            .increment("postgres.annotation.prefix_nodes", self.nodes % INTERVAL);
        self.observer
            .increment("postgres.annotation.prefix_containers", self.containers);
        if self.nodes >= INTERVAL && !self.nodes.is_multiple_of(INTERVAL) {
            self.emit();
        }
    }
}

#[cfg(test)]
mod tests;
