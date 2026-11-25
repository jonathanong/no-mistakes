use std::cell::Cell;

thread_local! {
    static BUILDS: Cell<usize> = const { Cell::new(0) };
}

pub(crate) fn note_build() {
    BUILDS.with(|builds| builds.set(builds.get() + 1));
}

pub(super) fn take_builds() -> usize {
    BUILDS.with(|builds| builds.replace(0))
}
