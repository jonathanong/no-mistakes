thread_local! {
    static CALL_PATH_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn record() {
    CALL_PATH_COUNT.with(|count| count.set(count.get() + 1));
}

pub(crate) fn take_call_path_count() -> usize {
    CALL_PATH_COUNT.with(|count| count.replace(0))
}

macro_rules! record_call_path {
    () => {
        call_path_metrics_tests::record()
    };
}
