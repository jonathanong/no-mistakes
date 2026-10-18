// Test-only actual operation probes, independent of diagnostic observer counters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct Operations {
    pub captures: usize,
    pub restores: usize,
    pub fresh: usize,
}
thread_local! { static CURRENT: std::cell::Cell<Operations> = const { std::cell::Cell::new(Operations { captures: 0, restores: 0, fresh: 0 }) }; }
pub(super) fn reset() {
    CURRENT.set(Operations::default());
}
pub(super) fn snapshot() -> Operations {
    CURRENT.get()
}
pub(super) fn capture() {
    let mut value = CURRENT.get();
    value.captures += 1;
    CURRENT.set(value);
}
pub(super) fn restore() {
    let mut value = CURRENT.get();
    value.restores += 1;
    CURRENT.set(value);
}
pub(super) fn fresh() {
    let mut value = CURRENT.get();
    value.fresh += 1;
    CURRENT.set(value);
}
