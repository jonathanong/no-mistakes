#[test]
fn alternative_capture_restore_keeps_original_readers_and_replaces_arm_readers() {
    let files = FxHashMap::default();
    let mut evaluator = evaluator(&files);
    evaluator.scopes = vec![Scope::default(); 4].into();
    evaluator.captured_bindings = FxHashMap::from_iter([
        (1, FxHashMap::from_iter([("value".into(), 0)])),
        (2, FxHashMap::from_iter([("value".into(), 0)])),
        (3, FxHashMap::from_iter([("value".into(), 2)])),
    ]);
    evaluator.rebuild_captured_readers();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());

    // Simulate a prior arm's disposable frame, followed by a module-initial
    // frame restored with a different capture origin in the next arm.
    evaluator.remove_alternative_captured_readers(2);
    evaluator.captured_bindings.remove(&2);
    evaluator.captured_bindings.remove(&3);
    evaluator
        .captured_bindings
        .insert(2, FxHashMap::from_iter([("value".into(), 1)]));
    evaluator
        .captured_bindings
        .insert(3, FxHashMap::from_iter([("value".into(), 2)]));
    evaluator.add_alternative_captured_readers(2);

    assert_eq!(
        evaluator.captured_binding_readers,
        FxHashMap::from_iter([
            (
                0,
                FxHashMap::from_iter([("value".into(), FxHashSet::from_iter([1]))]),
            ),
            (
                1,
                FxHashMap::from_iter([("value".into(), FxHashSet::from_iter([2]))]),
            ),
            (
                2,
                FxHashMap::from_iter([("value".into(), FxHashSet::from_iter([3]))]),
            ),
        ])
    );
    assert!(!observer
        .snapshot()
        .work
        .contains_key("postgres.annotation.captured_reader_rebuilds"));
}

#[test]
fn compaction_remaps_existing_readers_without_full_reconstruction() {
    let mut outcomes = Vec::new();
    // Reuse the same evaluator specialization in both modes: diagnostics must
    // not select a different ownership transformation or SQL proof.
    for enabled in [false, true] {
        let files = FxHashMap::default();
        let mut evaluator = evaluator(&files);
        evaluator.scopes = vec![Scope::default(); 4].into();
        evaluator.captured_bindings = FxHashMap::from_iter([
            (0, FxHashMap::from_iter([("prefix".into(), 2)])),
            (2, FxHashMap::from_iter([("cycle".into(), 0)])),
            (3, FxHashMap::from_iter([("dropped".into(), 2)])),
        ]);
        evaluator.rebuild_captured_readers();
        // Preserve frame zero, move frame two, and drop frame three. This is the
        // exact forward relation produced by the injective compaction mapping.
        evaluator.captured_bindings = FxHashMap::from_iter([
            (0, FxHashMap::from_iter([("prefix".into(), 1)])),
            (1, FxHashMap::from_iter([("cycle".into(), 0)])),
        ]);
        let observer = crate::diagnostics::InvocationObserver::new(true);
        crate::diagnostics::with_observer(enabled.then(|| observer.clone()), || {
            evaluator.remap_compacted_binding_indexes(1, &FxHashMap::from_iter([(2, 1)]), true);
            let remapped = evaluator.captured_binding_readers.clone();
            evaluator.remap_compacted_binding_indexes(1, &FxHashMap::default(), false);
            assert_eq!(remapped, evaluator.captured_binding_readers);
            let work = observer.snapshot().work;
            assert!(!work.contains_key("postgres.annotation.captured_reader_rebuilds"));
            if enabled {
                assert_eq!(
                    work["postgres.annotation.captured_reader_names_retained"],
                    2
                );
            } else {
                assert!(
                    work.is_empty(),
                    "disabled mode must not record remap counters"
                );
            }
            evaluator.rebuild_captured_readers();
            assert_eq!(remapped, evaluator.captured_binding_readers);
            if enabled {
                assert_eq!(
                    observer.snapshot().work["postgres.annotation.captured_reader_rebuilds"],
                    1
                );
            } else {
                assert!(observer.snapshot().work.is_empty());
            }
            // The exact reverse index must still propagate a proven SQL value to its reader.
            let value = Value::Prefix("SELECT 1".into(), true, None);
            evaluator.write_captured_binding(1, "prefix", &value);
            assert!(evaluator.scopes[0]["prefix"] == value);
            assert!(evaluator.scopes[1]["prefix"] == value);
            outcomes.push((
                remapped,
                evaluator.scopes[0]["prefix"].clone(),
                evaluator.scopes[1]["prefix"].clone(),
            ));
        });
    }
    assert!(outcomes[0] == outcomes[1]);
}
