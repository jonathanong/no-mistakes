use std::cell::Cell;
thread_local! { static COUNTS: Cell<(usize, usize, usize)> = const { Cell::new((0, 0, 0)) }; }
thread_local! { static ORIGINS: Cell<usize> = const { Cell::new(0) }; }
macro_rules! extend_capture_edges {
    ($found:expr, $origins:expr) => {
        $found.pending_env.extend($origins.values().map(|origin| {
            $crate::codebase::postgres::query_annotation::evaluate::alternatives::reachable::scan_hooks::origin();
            *origin
        }));
    };
}
pub(super) fn origin() {
    ORIGINS.with(|cell| cell.set(cell.get() + 1));
}
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) struct OriginGuard(
    usize,
);
impl OriginGuard {
    pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn new() -> Self {
        Self(ORIGINS.with(|cell| cell.replace(0)))
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn visits(
        &self,
    ) -> usize {
        ORIGINS.with(Cell::get)
    }
}
impl Drop for OriginGuard {
    fn drop(&mut self) {
        ORIGINS.with(|cell| cell.set(self.0));
    }
}
macro_rules! record_raw_binding { () => { $crate::codebase::postgres::query_annotation::evaluate::alternatives::reachable::scan_hooks::binding(); }; }
macro_rules! record_raw_slot { () => { $crate::codebase::postgres::query_annotation::evaluate::alternatives::reachable::scan_hooks::slot(); }; }
macro_rules! record_raw_value { () => { $crate::codebase::postgres::query_annotation::evaluate::alternatives::reachable::scan_hooks::value(); }; }
pub(super) fn binding() {
    COUNTS.with(|cell| {
        let (a, b, c) = cell.get();
        cell.set((a + 1, b, c));
    });
}
pub(super) fn slot() {
    COUNTS.with(|cell| {
        let (a, b, c) = cell.get();
        cell.set((a, b + 1, c));
    });
}
pub(super) fn value() {
    COUNTS.with(|cell| {
        let (a, b, c) = cell.get();
        cell.set((a, b, c + 1));
    });
}
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) struct Guard(
    (usize, usize, usize),
);
impl Guard {
    pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn new() -> Self {
        Self(COUNTS.with(|cell| cell.replace((0, 0, 0))))
    }
    pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn counts(
        &self,
    ) -> (usize, usize, usize) {
        COUNTS.with(Cell::get)
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        COUNTS.with(|cell| cell.set(self.0));
    }
}
