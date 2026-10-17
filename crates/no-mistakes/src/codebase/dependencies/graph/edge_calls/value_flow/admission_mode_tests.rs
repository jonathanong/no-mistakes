use super::*;

#[test]
fn admission_mode_is_nested_panic_safe_and_captured_per_request() {
    let old = with_original_admission(true, || {
        assert!(
            std::panic::catch_unwind(|| with_original_admission(false, || panic!(
                "saved mode unwind"
            )))
            .is_err()
        );
        ParameterCache::default()
    });
    let new = ParameterCache::default();
    assert!(old.admission_probe.original_admission());
    assert!(!new.admission_probe.original_admission());
    std::thread::scope(|scope| {
        scope.spawn(|| assert!(old.admission_probe.original_admission()));
        scope.spawn(|| assert!(!new.admission_probe.original_admission()));
    });
}
