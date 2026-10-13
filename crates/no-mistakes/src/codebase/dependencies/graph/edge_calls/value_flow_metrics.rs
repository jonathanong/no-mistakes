thread_local! {
    static LAST_EVALUATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
pub(super) fn record(evaluations: usize) {
    LAST_EVALUATIONS.set(evaluations);
}
pub fn last_evaluations() -> usize {
    LAST_EVALUATIONS.get()
}
