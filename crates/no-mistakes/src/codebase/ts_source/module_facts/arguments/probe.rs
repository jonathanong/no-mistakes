//! Test-only count of argument projections actually constructed.
macro_rules! record_projected_call {
    () => {
        $crate::codebase::ts_source::module_facts::arguments::probe::record()
    };
}

thread_local! {
    static PROJECTED_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn record() {
    PROJECTED_CALLS.with(|count| count.set(count.get() + 1));
}

pub(crate) fn projected_calls() -> usize {
    PROJECTED_CALLS.with(std::cell::Cell::get)
}

pub(crate) fn reset_projected_calls() {
    PROJECTED_CALLS.with(|count| count.set(0));
}
