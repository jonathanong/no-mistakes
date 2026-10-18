use crate::diagnostics::InvocationObserver;
use std::sync::Arc;

// Run-local bookkeeping; no persistent evaluator cache, clocks, or disabled
// observer maps. Capture TLS once and aggregate only after the run exits.
pub(super) struct Counters {
    observer: Option<Arc<InvocationObserver>>,
    plan: usize,
    tasks: u64,
    captures: u64,
    restores: u64,
    fresh: u64,
}
impl Counters {
    pub(super) fn new() -> Self {
        Self {
            observer: crate::diagnostics::current(),
            plan: 0,
            tasks: 0,
            captures: 0,
            restores: 0,
            fresh: 0,
        }
    }
    pub(super) fn plan(&mut self, value: usize) {
        self.plan = value;
    }
    pub(super) fn task(&mut self) {
        self.tasks += 1;
    }
    pub(super) fn capture(&mut self) {
        self.captures += 1;
    }
    pub(super) fn restore(&mut self) {
        self.restores += 1;
    }
    pub(super) fn fresh_rebuild(&mut self) {
        self.fresh += 1;
    }
}
impl Drop for Counters {
    fn drop(&mut self) {
        let Some(observer) = &self.observer else {
            return;
        };
        observer.increment(
            match self.plan {
                0 => "postgres.annotation.run.entrypoints.zero",
                1 => "postgres.annotation.run.entrypoints.one",
                _ => "postgres.annotation.run.entrypoints.many",
            },
            1,
        );
        for (name, count) in [
            ("postgres.annotation.run.entrypoints", self.tasks),
            ("postgres.annotation.run.checkpoint_captures", self.captures),
            ("postgres.annotation.run.checkpoint_restores", self.restores),
            ("postgres.annotation.run.fresh_index_rebuilds", self.fresh),
        ] {
            if count != 0 {
                observer.increment(name, count);
            }
        }
    }
}

#[cfg(test)]
#[path = "counters_tests.rs"]
mod tests;
