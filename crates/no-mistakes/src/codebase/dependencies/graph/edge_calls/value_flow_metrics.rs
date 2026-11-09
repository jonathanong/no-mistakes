thread_local! {
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

thread_local! { static LAST_MODULE_PROVIDER_CHECKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
pub(super) fn record_module_provider_checks(count: usize) {
    LAST_MODULE_PROVIDER_CHECKS.set(count);
}
pub fn module_provider_checks() -> usize {
    LAST_MODULE_PROVIDER_CHECKS.get()
}

thread_local! { static LAST_BUILTIN_CALLBACK_SITES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
pub(super) fn record_builtin_callback_sites(count: usize) {
    LAST_BUILTIN_CALLBACK_SITES.set(count);
}
#[cfg(test)]
#[path = "value_flow_metrics_tests.rs"]
pub(crate) mod tests;

thread_local! { static LAST_CALLBACK_OWNERSHIP_CACHE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
pub(super) fn record_callback_ownership_cache(initialized: bool) {
    LAST_CALLBACK_OWNERSHIP_CACHE.set(initialized);
}
#[cfg(test)]
pub fn callback_ownership_cache_initialized() -> bool {
    LAST_CALLBACK_OWNERSHIP_CACHE.get()
}
