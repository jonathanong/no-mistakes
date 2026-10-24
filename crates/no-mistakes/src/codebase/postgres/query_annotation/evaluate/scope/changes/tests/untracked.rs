use super::*;

#[test]
fn recording_after_discard_cannot_recover_ancestry_or_allocate_a_cold_journal() {
    let initial = scope();
    let mut changed = initial.clone();
    changed.insert("tracked".into(), Value::Arguments(9));
    assert!(!changed.1.is_empty());
    changed.1.clear();
    // A discarded journal has no proven origin. A write name alone cannot
    // rebuild that proof or silently turn the cold path into an allocation.
    changed.1.record("tracked");
    assert!(changed.1.is_empty());
    assert!(changed.changed_binding_names(&initial).is_none());
    assert!(changed["tracked"] == Value::Arguments(9));

    let mut cold = Changes::default();
    cold.record("untracked");
    assert!(cold.is_empty());
    assert!(cold.since(&initial).is_none());
}
