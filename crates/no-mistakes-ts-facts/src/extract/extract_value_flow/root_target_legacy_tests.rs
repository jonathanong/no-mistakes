thread_local! {
    pub(in super::super) static LEGACY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub(in super::super) static LEGACY_IDENTIFIER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
