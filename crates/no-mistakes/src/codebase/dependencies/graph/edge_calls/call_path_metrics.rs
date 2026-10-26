#[cfg(not(test))]
macro_rules! record_call_path {
    () => {};
}

#[cfg(test)]
#[macro_use]
#[path = "call_path_metrics_tests.rs"]
pub(crate) mod call_path_metrics_tests;
