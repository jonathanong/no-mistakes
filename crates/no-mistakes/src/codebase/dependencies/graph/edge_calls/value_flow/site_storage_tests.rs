use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

thread_local! {
    static CLONING: Cell<bool> = const { Cell::new(false) };
    static CLONES: Cell<usize> = const { Cell::new(0) };
    static ORIGINAL: Cell<bool> = const { Cell::new(false) };
    static LAST: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
}

/// The final CallPass captures its mode before Rayon; workers share its counters.
#[derive(Default)]
pub(super) struct Probe {
    mode: Mode,
    cloning: CloneMode,
    whole_clones: AtomicUsize,
    vectors: AtomicUsize,
    scalars: AtomicUsize,
}
struct Mode(bool);
struct CloneMode(bool);
impl Default for CloneMode {
    fn default() -> Self {
        Self(CLONING.get())
    }
}
impl Default for Mode {
    fn default() -> Self {
        Self(ORIGINAL.get())
    }
}
impl Probe {
    pub(super) fn cloning(&self) -> bool {
        self.cloning.0
    }
    pub(super) fn whole_clone(&self) {
        self.whole_clones.fetch_add(1, Ordering::Relaxed);
    }
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
        CLONES.set(self.whole_clones.load(Ordering::Relaxed));
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

pub(in super::super) fn with_cloning<T>(cloning: bool, run: impl FnOnce() -> T) -> (T, usize) {
    struct Restore(bool, usize);
    impl Drop for Restore {
        fn drop(&mut self) {
            CLONING.set(self.0);
            CLONES.set(self.1);
        }
    }
    let _restore = Restore(CLONING.replace(cloning), CLONES.replace(0));
    let result = run();
    (result, CLONES.get())
}
