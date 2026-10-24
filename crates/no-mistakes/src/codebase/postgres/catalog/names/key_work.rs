//! Count catalog-key processing for operation-count tests and instrumented benchmarks.
thread_local! {
    static KEY_SPLITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn count() -> usize {
    KEY_SPLITS.with(std::cell::Cell::get)
}

pub(super) fn record() {
    let next = count() + 1;
    KEY_SPLITS.with(|count| count.set(next));
}
