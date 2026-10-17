thread_local! {
    static GLOBAL_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(super) fn record() {
    GLOBAL_VISITS.with(|visits| visits.set(visits.get() + 1));
}

pub(super) fn reset() {
    GLOBAL_VISITS.with(|visits| visits.set(0));
}

pub(super) fn count() -> usize {
    GLOBAL_VISITS.with(std::cell::Cell::get)
}
