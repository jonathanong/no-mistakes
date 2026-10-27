// Test hooks expand to no production code or argument evaluation.
macro_rules! probe {
    () => {};
}
macro_rules! extend_origins {
    ($found:expr, $origins:expr) => {
        $found.pending_env.extend($origins.values().copied());
    };
}
macro_rules! legacy_dispatch {
    ($($tokens:tt)*) => {};
}
