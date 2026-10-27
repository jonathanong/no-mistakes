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
thread_local! { static LIVE_BINDINGS: Cell<usize> = const { Cell::new(0) }; }
pub(super) fn record_live_binding() {
    LIVE_BINDINGS.set(LIVE_BINDINGS.get() + 1);
}
pub(super) fn measure_live_bindings<T>(run: impl FnOnce() -> T) -> (T, usize) {
    struct Restore(usize);
    impl Drop for Restore {
        fn drop(&mut self) {
            LIVE_BINDINGS.set(self.0);
        }
    }
    let _restore = Restore(LIVE_BINDINGS.replace(0));
    let result = run();
    (result, LIVE_BINDINGS.get())
}
macro_rules! record_live_binding {
    () => {
        $crate::codebase::postgres::query_annotation::evaluate::effects::hooks::record_live_binding(
        );
    };
}
pub(super) use record_live_binding;
