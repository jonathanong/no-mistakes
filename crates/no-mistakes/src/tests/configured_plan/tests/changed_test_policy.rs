#[test]
fn structured_changed_test_policy_uses_prepared_and_ignored_test_sets() {
    let changed = PathBuf::from("src/example.test.ts");
    let discovered = HashSet::from([changed.clone()]);
    let ignored = vec![HashSet::from([changed.clone()])];
    let empty = HashSet::new();

    assert!(super::dep_triggers::structured_trigger_skips_changed_test(
        &changed,
        &discovered,
        &[],
        None,
    ));
    assert!(super::dep_triggers::structured_trigger_skips_changed_test(
        &changed,
        &discovered,
        &[],
        Some(false),
    ));
    assert!(!super::dep_triggers::structured_trigger_skips_changed_test(
        &changed,
        &discovered,
        &[],
        Some(true),
    ));
    assert!(super::dep_triggers::structured_trigger_skips_changed_test(
        &changed, &empty, &ignored, None,
    ));
    assert!(!super::dep_triggers::structured_trigger_skips_changed_test(
        &changed,
        &empty,
        &ignored,
        Some(true),
    ));
    assert!(!super::dep_triggers::structured_trigger_skips_changed_test(
        &PathBuf::from("src/source.ts"),
        &discovered,
        &ignored,
        None,
    ));
}
