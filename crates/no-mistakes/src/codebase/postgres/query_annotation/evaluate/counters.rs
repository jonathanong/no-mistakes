//! Per-root totals for annotation work counters.
//!
//! Inner iterations only bump integers. Each evaluator root flushes once
//! through [`crate::diagnostics::with_current`], so workers do not clone the
//! shared observer `Arc` or take the counter extension map on every walk or
//! slot copy. Diagnostics-off runs return before this state is touched.

use std::cell::RefCell;

const WALKS: &str = "postgres.annotation.reachability_walks";
const SLOTS: &str = "postgres.annotation.incoming_dense_slots_copied";
const REBUILDS: &str = "postgres.annotation.captured_reader_rebuilds";
const NAMES: &str = "postgres.annotation.captured_reader_names_retained";

#[derive(Clone, Copy, Default)]
struct Totals {
    walks: u64,
    slots: u64,
    slots_seen: bool,
    rebuilds: u64,
    names: u64,
    names_seen: bool,
}

impl Totals {
    fn is_empty(self) -> bool {
        self.walks == 0 && !self.slots_seen && self.rebuilds == 0 && !self.names_seen
    }

    fn flush(self) {
        if self.is_empty() {
            return;
        }
        crate::diagnostics::with_current(|observer| {
            if self.walks != 0 {
                observer.increment(WALKS, self.walks);
            }
            if self.slots_seen {
                observer.increment(SLOTS, self.slots);
            }
            if self.rebuilds != 0 {
                observer.increment(REBUILDS, self.rebuilds);
            }
            if self.names_seen {
                observer.increment(NAMES, self.names);
            }
        });
    }
}

thread_local! {
    static STACK: RefCell<Vec<Totals>> = const { RefCell::new(Vec::new()) };
}

/// Accumulator installed around one evaluator root on this thread.
pub(super) struct RootCounters {
    active: bool,
}

impl RootCounters {
    pub(super) fn begin() -> Self {
        if !crate::diagnostics::work_enabled() {
            return Self { active: false };
        }
        STACK.with(|stack| stack.borrow_mut().push(Totals::default()));
        Self { active: true }
    }
}

impl Drop for RootCounters {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let totals = STACK.with(|stack| {
            stack
                .borrow_mut()
                .pop()
                .expect("annotation counter batch must be active")
        });
        totals.flush();
    }
}

fn accumulate(update: impl FnOnce(&mut Totals)) -> bool {
    STACK.with(|stack| {
        let mut stack = stack.borrow_mut();
        let Some(totals) = stack.last_mut() else {
            return false;
        };
        update(totals);
        true
    })
}

#[inline]
fn publish(metric: &'static str, amount: u64, update: impl FnOnce(&mut Totals)) {
    if !crate::diagnostics::work_enabled() {
        return;
    }
    if accumulate(update) {
        return;
    }
    crate::diagnostics::with_current(|observer| observer.increment(metric, amount));
}

#[inline]
pub(super) fn note_walk() {
    publish(WALKS, 1, |totals| totals.walks += 1);
}

#[inline]
pub(super) fn note_slots(copied: u64) {
    publish(SLOTS, copied, |totals| {
        totals.slots += copied;
        totals.slots_seen = true;
    });
}

#[inline]
pub(super) fn note_rebuild() {
    publish(REBUILDS, 1, |totals| totals.rebuilds += 1);
}

#[inline]
pub(super) fn note_names(retained: u64) {
    publish(NAMES, retained, |totals| {
        totals.names += retained;
        totals.names_seen = true;
    });
}

#[cfg(test)]
#[path = "counters/tests.rs"]
mod tests;
