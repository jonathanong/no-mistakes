macro_rules! identity {
    ($a:expr, $b:expr) => {
        std::ptr::eq($a, $b)
    };
}
macro_rules! fallback {
    () => {};
}
pub(super) use super::prefix_order::compare as prefix;
macro_rules! sql_component {
    ($a:expr, $b:expr) => {
        $a == $b
    };
}
pub(super) use {fallback, identity, sql_component};

pub(super) const fn memoize() -> bool {
    true
}
