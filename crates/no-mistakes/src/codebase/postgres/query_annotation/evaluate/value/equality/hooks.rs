macro_rules! identity {
    ($a:expr, $b:expr) => {
        std::ptr::eq($a, $b)
    };
}
macro_rules! fallback {
    () => {};
}
pub(super) use {fallback, identity};
