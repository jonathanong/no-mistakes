use std::cell::Cell;

macro_rules! yield_symbol_index_reentry {
    () => {
        $crate::codebase::dependencies::shared_symbol_index_reentry_tests::yield_once();
    };
}

thread_local! {
    static ARMED: Cell<bool> = const { Cell::new(false) };
    static YIELDS: Cell<usize> = const { Cell::new(0) };
}

pub(crate) fn with_one_yield<T>(run: impl FnOnce() -> T) -> (T, usize) {
    struct Restore(bool, usize);
    impl Drop for Restore {
        fn drop(&mut self) {
            ARMED.set(self.0);
            YIELDS.set(self.1);
        }
    }
    let _restore = Restore(ARMED.replace(true), YIELDS.replace(0));
    let value = run();
    (value, YIELDS.get())
}

pub(crate) fn yield_once() {
    if ARMED.replace(false) {
        YIELDS.set(YIELDS.get() + 1);
        // Expose a queued outer report while its sibling initializes the index.
        let _ = rayon::yield_local();
    }
}
