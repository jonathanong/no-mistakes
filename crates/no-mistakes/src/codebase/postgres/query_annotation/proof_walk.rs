//! Counts append-callback proof walks that still have a chance to change the proof.
thread_local! {
    static WALKS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

pub(super) fn record() {
    WALKS.with(|walks| walks.set(walks.get() + 1));
}

#[cfg_attr(
    not(test),
    expect(dead_code, reason = "tests read the proof-walk counter")
)]
pub(super) fn count() -> u32 {
    WALKS.with(std::cell::Cell::get)
}
