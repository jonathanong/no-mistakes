//! Test-only work observations for an isolated synchronous SQL extraction.
use std::cell::Cell;
#[derive(Clone, Copy, Default, Debug)]
pub(crate) struct Counts {
    pub full_walks: usize,
    pub general_fact_collections: usize,
    pub statement_extractions: usize,
    pub full_projections: usize,
    pub conflict_projections: usize,
}
thread_local! { static COUNTS: Cell<Option<Counts>> = const { Cell::new(None) }; }
pub(crate) fn begin() {
    COUNTS.set(Some(Counts::default()));
}
pub(crate) fn finish() -> Counts {
    COUNTS.take().expect("work observation active")
}
fn record(update: impl FnOnce(&mut Counts)) {
    COUNTS.with(|counts| {
        if let Some(mut value) = counts.get() {
            update(&mut value);
            counts.set(Some(value));
        }
    });
}
pub(crate) fn full_walk() {
    record(|counts| counts.full_walks += 1);
}
pub(crate) fn full_projection() {
    record(|counts| counts.full_projections += 1);
}
pub(crate) fn conflict_projection() {
    record(|counts| counts.conflict_projections += 1);
}

pub(crate) fn statement_extraction() {
    record(|counts| counts.statement_extractions += 1);
}

pub(crate) fn general_fact_collection() {
    record(|counts| counts.general_fact_collections += 1);
}
