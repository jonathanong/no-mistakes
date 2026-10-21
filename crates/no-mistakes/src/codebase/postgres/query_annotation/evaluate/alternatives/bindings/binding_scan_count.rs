// Test hooks expand to no production code or argument evaluation.
macro_rules! record_join_binding {
    () => {};
}
macro_rules! legacy_join {
    ($joined:expr, $current:expr, $original:expr, $whole_sql:expr) => {};
}
