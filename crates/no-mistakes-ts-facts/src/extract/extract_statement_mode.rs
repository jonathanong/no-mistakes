// Test-only legacy oracle, captured by the request cache before statement traversal.
thread_local! {
    static ORIGINAL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
#[derive(Clone, Copy)]
pub(super) struct Mode(bool);
impl Default for Mode {
    fn default() -> Self {
        Self(ORIGINAL.get())
    }
}
impl Mode {
    pub(super) fn original(self) -> bool {
        self.0
    }
}
pub fn with_original_statements<T>(original: bool, run: impl FnOnce() -> T) -> T {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            ORIGINAL.set(self.0);
        }
    }
    let _restore = Restore(ORIGINAL.replace(original));
    run()
}

#[test]
fn request_mode_restores_after_unwind_and_crosses_workers_by_value() {
    assert!(!Mode::default().original());
    let original = with_original_statements(true, || {
        let mode = Mode::default();
        assert!(
            std::panic::catch_unwind(|| with_original_statements(false, || panic!("restore")))
                .is_err()
        );
        assert!(Mode::default().original());
        mode
    });
    let optimized = Mode::default();
    assert!(!optimized.original());
    std::thread::scope(|scope| {
        scope.spawn(|| assert!(original.original()));
        scope.spawn(|| assert!(!optimized.original()));
    });
}
