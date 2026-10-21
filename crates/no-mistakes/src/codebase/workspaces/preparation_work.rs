//! Thread-local work accounting disappears from production builds.
use std::cell::Cell;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Counts {
    pub(crate) visible_paths: usize,
    pub(crate) package_dirs: usize,
    pub(crate) ancestor_lookups: usize,
}
thread_local! { static WORK: Cell<Counts> = Cell::new(Counts::default()); }
pub(crate) fn reset() {
    WORK.set(Counts::default());
}
pub(crate) fn counts() -> Counts {
    WORK.get()
}
pub(crate) fn visible_path() {
    WORK.update(|mut c| {
        c.visible_paths += 1;
        c
    });
}
pub(crate) fn package_dir() {
    WORK.update(|mut c| {
        c.package_dirs += 1;
        c
    });
}
pub(crate) fn ancestor_lookup() {
    WORK.update(|mut c| {
        c.ancestor_lookups += 1;
        c
    });
}

macro_rules! record_preparation {
    ($kind:ident) => {
        $crate::codebase::workspaces::preparation_work::$kind()
    };
}
