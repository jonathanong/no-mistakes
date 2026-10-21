//! Thread-local work accounting disappears from production builds.
use std::cell::Cell;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Counts {
    pub(super) package_dirs: usize,
    pub(super) ancestor_lookups: usize,
}
thread_local! { static WORK: Cell<Counts> = Cell::new(Counts::default()); }
pub(super) fn reset() {
    WORK.set(Counts::default());
}
pub(super) fn counts() -> Counts {
    WORK.get()
}
pub(super) fn package_dir() {
    WORK.update(|mut c| {
        c.package_dirs += 1;
        c
    });
}
pub(super) fn ancestor_lookup() {
    WORK.update(|mut c| {
        c.ancestor_lookups += 1;
        c
    });
}

macro_rules! record_preparation {
    ($kind:ident) => {
        $crate::codebase::rules::production_dependency_declarations::discovery::preparation_work::$kind()
    };
}
