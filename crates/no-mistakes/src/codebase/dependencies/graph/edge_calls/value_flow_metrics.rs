thread_local! {
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
