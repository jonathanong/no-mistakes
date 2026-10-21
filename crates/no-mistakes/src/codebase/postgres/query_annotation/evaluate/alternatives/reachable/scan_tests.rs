use std::cell::Cell;
thread_local! { static COUNTS: Cell<(usize, usize, usize)> = const { Cell::new((0, 0, 0)) }; }
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
