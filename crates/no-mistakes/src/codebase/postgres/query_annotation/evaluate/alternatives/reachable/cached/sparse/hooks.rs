// Test hooks expand to no production code or argument evaluation.
macro_rules! probe {
    () => {};
}
macro_rules! legacy_dispatch {
    ($($tokens:tt)*) => {};
}
