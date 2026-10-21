macro_rules! root_values {
    ($scope:expr) => {
        ($scope).values().cloned()
    };
}
pub(super) use root_values;
