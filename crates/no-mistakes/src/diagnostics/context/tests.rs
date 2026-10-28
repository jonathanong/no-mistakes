use std::cell::Cell;

thread_local! {
    static CURRENT_LOOKUPS: Cell<usize> = const { Cell::new(0) };
}

pub(super) fn record_current_lookup() {
    CURRENT_LOOKUPS.with(|lookups| lookups.set(lookups.get() + 1));
}

pub(crate) fn current_lookup_count() -> usize {
    CURRENT_LOOKUPS.with(Cell::get)
}
