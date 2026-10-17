use super::*;

#[test]
fn identity_frames_with_changed_binding_values_still_prune_arena_and_freshness() {
    let mut scopes = vec![Scope::from(FxHashMap::from_iter([(
        "arg".into(),
        Value::Arguments(7),
    )]))];
    // Branch restoration can replace values while every frame ID stays fixed.
    // Capture identity must not short-circuit the rest of compaction.
    scopes[0].insert("arg".into(), Value::Arguments(8));
    let mut objects = FxHashMap::from_iter([
        (7, vec![Value::Arguments(7)]),
        (8, vec![Value::Arguments(8)]),
        (99, vec![Value::Arguments(99)]),
    ]);
    let mut extras = FxHashMap::default();
    let mut mapped = FxHashMap::default();
    let mut fresh = FxHashMap::from_iter([
        (0, FxHashSet::from_iter(["arg".into()])),
        (9, FxHashSet::from_iter(["stale".into()])),
    ]);
    let mut captured = FxHashMap::from_iter([(0, FxHashMap::default())]);
    let result = compact_from_roots(
        &mut scopes,
        Roots {
            environments: &[0],
            arguments: &[],
            preserved: 1,
            transient: &[],
        },
        &mut [],
        &mut mapped,
        ArenaMut {
            objects: &mut objects,
            extras: &mut extras,
        },
        &mut fresh,
        &mut captured,
    );
    assert!(result.indices.is_empty());
    assert!(!result.captured_changed);
    assert_eq!(result.arguments, FxHashSet::from_iter([8]));
    assert!(scopes[0]["arg"] == Value::Arguments(8));
    assert_eq!(
        objects.keys().copied().collect::<FxHashSet<_>>(),
        FxHashSet::from_iter([8])
    );
    assert!(fresh.contains_key(&0));
    assert!(!fresh.contains_key(&9));
}
