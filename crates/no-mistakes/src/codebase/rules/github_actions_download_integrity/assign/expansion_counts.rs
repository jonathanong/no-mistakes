use std::cell::Cell;

thread_local! {
    static EXPANSIONS: Cell<usize> = const { Cell::new(0) };
    static BINDING_SCANS: Cell<usize> = const { Cell::new(0) };
}

pub(super) fn note() {
    EXPANSIONS.with(|count| count.set(count.get() + 1));
}

pub(super) fn count() -> usize {
    EXPANSIONS.with(Cell::get)
}

pub(super) fn note_binding_scan() {
    BINDING_SCANS.with(|count| count.set(count.get() + 1));
}

pub(super) fn binding_scans() -> usize {
    BINDING_SCANS.with(Cell::get)
}
