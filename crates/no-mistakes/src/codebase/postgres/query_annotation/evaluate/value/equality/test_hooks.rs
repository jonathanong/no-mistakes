use std::cell::Cell;
thread_local! {
    static LEGACY: Cell<bool> = const { Cell::new(false) };
    static FALLBACKS: Cell<usize> = const { Cell::new(0) };
}
pub(super) fn same_value(a: &super::Value, b: &super::Value) -> bool {
    !LEGACY.get() && std::ptr::eq(a, b)
}
pub(super) fn record_fallback() {
    FALLBACKS.set(FALLBACKS.get() + 1);
}
pub(super) fn measure<T>(legacy: bool, f: impl FnOnce() -> T) -> (T, usize) {
    struct Reset(bool);
    impl Drop for Reset {
        fn drop(&mut self) {
            LEGACY.set(self.0);
        }
    }
    let _reset = Reset(LEGACY.replace(legacy));
    FALLBACKS.set(0);
    let result = f();
    (result, FALLBACKS.get())
}
macro_rules! identity {
    ($a:expr, $b:expr) => {
        hooks::same_value($a, $b)
    };
}
macro_rules! fallback {
    () => {
        hooks::record_fallback()
    };
}
pub(super) use {fallback, identity};
