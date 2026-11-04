#[test]
fn absent_prepared_bindings_do_not_exempt_exports() {
    assert!(super::exports(&Default::default()).is_empty());
}
