use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

thread_local! {
    static ORIGINAL: Cell<bool> = const { Cell::new(false) };
    static LAST: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
}

/// The final CallPass captures its mode before Rayon; workers share its counters.
#[derive(Default)]
pub(super) struct Probe {
    mode: Mode,
    vectors: AtomicUsize,
    scalars: AtomicUsize,
}
struct Mode(bool);
impl Default for Mode {
    fn default() -> Self {
        Self(ORIGINAL.get())
    }
}
impl Probe {
    pub(super) fn original(&self) -> bool {
        self.mode.0
    }
    pub(super) fn vector(&self) {
        self.vectors.fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn scalar(&self) {
        self.scalars.fetch_add(1, Ordering::Relaxed);
    }
    pub(super) fn finish(&self) {
        LAST.set((
            self.vectors.load(Ordering::Relaxed),
            self.scalars.load(Ordering::Relaxed),
        ));
    }
}
pub(in super::super) fn with_original<T>(original: bool, run: impl FnOnce() -> T) -> T {
    struct Restore(bool, (usize, usize));
    impl Drop for Restore {
        fn drop(&mut self) {
            ORIGINAL.set(self.0);
            LAST.set(self.1);
        }
    }
    let _restore = Restore(ORIGINAL.replace(original), LAST.replace((0, 0)));
    run()
}
pub(in super::super) fn last() -> (usize, usize) {
    LAST.get()
}
