thread_local! {
    static WALKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(super) fn record() {
    WALKS.with(|count| count.set(count.get() + 1));
}

pub(super) fn reset() {
    WALKS.with(|count| count.set(0));
}

pub(super) fn count() -> usize {
    WALKS.with(std::cell::Cell::get)
}

macro_rules! record_sql_walk {
    () => {
        work_probe::record()
    };
}
