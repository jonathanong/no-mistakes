use std::cell::Cell;

thread_local! {
    static VISIBLE_MAP_BUILDS: Cell<Option<usize>> = const { Cell::new(None) };
    static BUILDER_WORK: Cell<Option<(usize, usize)>> = const { Cell::new(None) };
}

pub(in crate::codebase::postgres::statements::bounds) fn begin() {
    VISIBLE_MAP_BUILDS.set(Some(0));
}

pub(in crate::codebase::postgres::statements::bounds) fn finish() -> usize {
    VISIBLE_MAP_BUILDS
        .take()
        .expect("visible map work observation active")
}

pub(super) fn visible_map_build() {
    VISIBLE_MAP_BUILDS.with(|builds| {
        if let Some(count) = builds.get() {
            builds.set(Some(count + 1));
        }
    });
}

pub(in crate::codebase::postgres::statements::bounds) fn begin_builder() {
    BUILDER_WORK.set(Some((0, 0)));
}

pub(in crate::codebase::postgres::statements::bounds) fn finish_builder() -> (usize, usize) {
    BUILDER_WORK
        .take()
        .expect("builder work observation active")
}

pub(super) fn builder_materialization() {
    BUILDER_WORK.with(|work| {
        if let Some((materializations, adds)) = work.get() {
            work.set(Some((materializations + 1, adds)));
        }
    });
}

pub(super) fn builder_relation_add() {
    BUILDER_WORK.with(|work| {
        if let Some((materializations, adds)) = work.get() {
            work.set(Some((materializations, adds + 1)));
        }
    });
}
