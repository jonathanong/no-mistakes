thread_local! {
    pub(super) static PATH_DESCENTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
