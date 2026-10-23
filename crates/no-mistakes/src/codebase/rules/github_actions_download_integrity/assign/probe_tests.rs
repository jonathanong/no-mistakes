use std::cell::Cell;

thread_local! {
    static EXPANSIONS: Cell<usize> = const { Cell::new(0) };
    static BINDING_SCANS: Cell<usize> = const { Cell::new(0) };
}

pub(crate) fn note() {
    EXPANSIONS.with(|count| count.set(count.get() + 1));
}

pub(crate) fn count() -> usize {
    EXPANSIONS.with(Cell::get)
}

pub(crate) fn note_binding_scan() {
    BINDING_SCANS.with(|count| count.set(count.get() + 1));
}

pub(crate) fn binding_scans() -> usize {
    BINDING_SCANS.with(Cell::get)
}
