use super::*;

#[test]
fn dense_join_copies_only_new_and_changed_slots() {
    let original = (0..64)
        .map(|index| Value::Prefix(format!("SELECT {index}"), true, None))
        .collect::<Vec<_>>();
    let first = FxHashMap::from_iter([(7, original.clone())]);
    let mut joined = FxHashMap::default();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());

    objects(&mut joined, &first);
    objects(&mut joined, &first);
    assert_eq!(
        observer.snapshot().work["postgres.annotation.incoming_dense_slots_copied"],
        64,
        "the second, identical arm borrows all existing slots"
    );

    let mut changed = original;
    changed[3] = Value::Unknown;
    objects(&mut joined, &FxHashMap::from_iter([(7, changed)]));
    assert_eq!(
        observer.snapshot().work["postgres.annotation.incoming_dense_slots_copied"],
        65,
        "only the changed arm slot is copied"
    );
    assert!(matches!(&joined[&7][3], Value::Possible(values)
        if values.contains(&Value::Prefix("SELECT 3".into(), true, None))
            && values.contains(&Value::Unknown)));
    assert!(matches!(&joined[&7][4], Value::Prefix(text, true, None)
        if text == "SELECT 4"));
}

#[test]
fn divergent_shape_remains_unknown_after_later_single_slot_arms() {
    let mut joined = FxHashMap::from_iter([(7, vec![Value::Primitive])]);
    objects(
        &mut joined,
        &FxHashMap::from_iter([(7, vec![Value::Primitive, Value::Primitive])]),
    );
    objects(
        &mut joined,
        &FxHashMap::from_iter([(7, vec![Value::Primitive])]),
    );
    assert_eq!(
        joined[&7],
        vec![Value::Failure("conditional-collection-shape")]
    );

    // Joining a previously failed arm into a fresh one must preserve its shape.
    let mut fresh = FxHashMap::from_iter([(7, vec![Value::Primitive])]);
    objects(&mut fresh, &joined);
    assert_eq!(fresh[&7], joined[&7]);
}
