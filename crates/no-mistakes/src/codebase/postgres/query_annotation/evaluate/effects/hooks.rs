macro_rules! root_values {
    ($scope:expr) => {
        ($scope).values().cloned()
    };
}
pub(super) use root_values;
macro_rules! record_live_binding {
    () => {};
}
pub(super) use record_live_binding;
