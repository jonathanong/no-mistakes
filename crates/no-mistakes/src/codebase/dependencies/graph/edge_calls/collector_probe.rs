//! Test-only, request-thread count of canonical call-collector entries.
macro_rules! record_call_collector_entry {
    () => {
        call_collector_probe::record()
    };
}

thread_local! {
    static CANONICAL_CALL_COLLECTOR_ENTRIES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn record() {
    CANONICAL_CALL_COLLECTOR_ENTRIES.with(|entries| entries.set(entries.get() + 1));
}

pub(crate) fn reset() {
    CANONICAL_CALL_COLLECTOR_ENTRIES.with(|entries| entries.set(0));
}

pub(crate) fn entries() -> usize {
    CANONICAL_CALL_COLLECTOR_ENTRIES.with(std::cell::Cell::get)
}
