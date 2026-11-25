use std::cell::Cell;

thread_local! {
    static PROGRAM_VISITS: Cell<usize> = const { Cell::new(0) };
}

pub(super) fn note_program_visit() {
    PROGRAM_VISITS.with(|visits| visits.set(visits.get() + 1));
}

pub(in crate::queue) fn take_program_visits() -> usize {
    PROGRAM_VISITS.with(|visits| visits.replace(0))
}
