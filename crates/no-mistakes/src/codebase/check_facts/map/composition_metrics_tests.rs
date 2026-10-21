use super::CheckFactMap;

thread_local! {
    static COMPOSITIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(super) fn record() {
    COMPOSITIONS.with(|count| count.set(count.get() + 1));
}

macro_rules! record_view_composition {
    () => {
        composition_metrics_tests::record()
    };
}

impl CheckFactMap {
    pub(crate) fn view_composition_count() -> usize {
        COMPOSITIONS.with(std::cell::Cell::get)
    }
}
