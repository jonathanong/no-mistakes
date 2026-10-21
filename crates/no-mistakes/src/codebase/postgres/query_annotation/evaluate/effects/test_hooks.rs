use std::cell::Cell;
thread_local! { static ROOT_CLONES: Cell<usize> = const { Cell::new(0) }; }
pub(super) fn record_root_clone() {
    ROOT_CLONES.set(ROOT_CLONES.get() + 1);
}
pub(super) fn measure<T>(run: impl FnOnce() -> T) -> (T, usize) {
    struct Restore(usize);
    impl Drop for Restore {
        fn drop(&mut self) {
            ROOT_CLONES.set(self.0);
        }
    }
    let _restore = Restore(ROOT_CLONES.replace(0));
    let result = run();
    (result, ROOT_CLONES.get())
}
macro_rules! root_values {
    ($scope:expr) => {
        ($scope)
            .values()
            .cloned()
            .inspect(|_| hooks::record_root_clone())
    };
}
pub(super) use root_values;
