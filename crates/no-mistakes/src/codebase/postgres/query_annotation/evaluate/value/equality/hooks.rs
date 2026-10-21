macro_rules! identity {
    ($a:expr, $b:expr) => {
        std::ptr::eq($a, $b)
    };
}
macro_rules! fallback {
    () => {};
}
macro_rules! prefix {
    ($a:expr, $b:expr, $c:expr, $a2:expr, $b2:expr, $c2:expr) => {
        $b == $b2 && $c == $c2 && $a == $a2
    };
}
pub(super) use {fallback, identity, prefix};
