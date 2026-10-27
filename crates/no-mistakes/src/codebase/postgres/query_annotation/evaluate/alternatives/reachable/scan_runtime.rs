macro_rules! record_raw_binding {
    () => {};
}
macro_rules! record_raw_slot {
    () => {};
}

macro_rules! record_raw_value {
    () => {};
}
macro_rules! extend_capture_edges {
    ($found:expr, $origins:expr) => {
        $found.pending_env.extend($origins.values().copied());
    };
}
