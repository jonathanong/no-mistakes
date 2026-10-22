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

macro_rules! owner_identity {
    ($a:expr,$b:expr) => {
        std::sync::Arc::ptr_eq($a, $b)
    };
}
macro_rules! owner_component {
    ($a:expr,$b:expr) => {
        $a == $b
    };
}
pub(super) use {owner_component, owner_identity};
