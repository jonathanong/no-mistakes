use std::cell::Cell;

thread_local! {
    pub(super) static WORK: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
}

pub(super) fn record_normalization() {
    WORK.with(|work| {
        let (paths, checks) = work.get();
        work.set((paths + 1, checks));
    });
}

pub(super) fn record_check() {
    WORK.with(|work| {
        let (paths, checks) = work.get();
        work.set((paths, checks + 1));
    });
}
