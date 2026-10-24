use std::cell::Cell;
thread_local! {
    static LEGACY: Cell<bool> = const { Cell::new(false) };
    static SQL_COMPARISONS: Cell<usize> = const { Cell::new(0) };
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
pub(super) fn compare_sql(a: &str, b: &str) -> bool {
    SQL_COMPARISONS.set(SQL_COMPARISONS.get() + 1);
    a == b
}
pub(super) fn measure_sql<T>(legacy: bool, f: impl FnOnce() -> T) -> (T, usize) {
    SQL_COMPARISONS.set(0);
    let (result, _) = measure(legacy, f);
    (result, SQL_COMPARISONS.get())
}
pub(super) fn legacy_prefix() -> bool {
    LEGACY.get()
}
macro_rules! sql_component {
    ($a:expr, $b:expr) => {
        hooks::compare_sql($a, $b)
    };
}
macro_rules! prefix {
    ($a:expr, $b:expr, $c:expr, $a2:expr, $b2:expr, $c2:expr) => {
        if hooks::legacy_prefix() {
            hooks::sql_component!($a, $a2) && $b == $b2 && $c == $c2
        } else {
            $crate::codebase::postgres::query_annotation::evaluate::value::equality::prefix_order::compare!($a, $b, $c, $a2, $b2, $c2)
        }
    };
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
pub(super) use {fallback, identity, prefix, sql_component};

pub(super) fn memoize() -> bool {
    !LEGACY.get()
}
