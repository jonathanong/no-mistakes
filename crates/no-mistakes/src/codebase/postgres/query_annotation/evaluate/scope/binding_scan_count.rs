// Test instrumentation expands to no production code or argument evaluation.
macro_rules! record_binding_scan {
    ($bindings:expr) => {};
}
