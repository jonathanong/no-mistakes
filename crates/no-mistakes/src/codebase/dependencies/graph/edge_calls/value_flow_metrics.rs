thread_local! {
    static LAST_NULLISH_WORK: std::cell::Cell<[usize; 15]> = const { std::cell::Cell::new([0; 15]) };
    static LAST_EXECUTION_STEPS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static LAST_DEPENDENCIES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static LAST_ORIGINS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static LAST_PARAMETERS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static LAST_EVALUATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
pub(super) fn record(evaluations: usize) {
    LAST_EVALUATIONS.set(evaluations);
}
pub fn last_evaluations() -> usize {
    LAST_EVALUATIONS.get()
}

pub(super) fn record_parameters(count: usize) {
    LAST_PARAMETERS.set(count);
}
pub fn parameter_agreements() -> usize {
    LAST_PARAMETERS.get()
}

pub(super) fn record_origins(count: usize) {
    LAST_ORIGINS.set(count);
}
pub fn origin_nodes() -> usize {
    LAST_ORIGINS.get()
}

pub(super) fn record_dependency_links(count: usize) {
    LAST_DEPENDENCIES.set(count);
}
pub fn dependency_links() -> usize {
    LAST_DEPENDENCIES.get()
}

#[path = "replacement_probe.rs"]
mod replacements;
pub use replacements::*;
pub(super) use replacements::{record_replacements, ReplacementLookupProbe};

pub(super) fn record_execution_steps(count: usize) {
    LAST_EXECUTION_STEPS.set(count);
}
pub fn execution_steps() -> usize {
    LAST_EXECUTION_STEPS.get()
}

pub(super) fn record_nullish_work(counts: [usize; 15]) {
    LAST_NULLISH_WORK.set(counts);
}
pub fn nullish_work() -> [usize; 15] {
    LAST_NULLISH_WORK.get()
}
