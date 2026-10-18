use super::*;
use crate::codebase::postgres::query_annotation::evaluate::{
    frame_arena::ScopeArena, scope::Scope,
};

#[test]
fn stable_handles_still_prune_dead_argument_and_freshness_state() {
    let mut scopes: ScopeArena = vec![Scope::from(FxHashMap::from_iter([(
        "arg".into(),
        Value::Arguments(8),
    )]))]
    .into();
    let root = scopes.ids().next().unwrap();
    let dead_frame = scopes.allocate(Scope::default());
    let mut objects = FxHashMap::from_iter([
        (7, vec![Value::Arguments(7)]),
        (8, vec![Value::Arguments(8)]),
        (99, vec![Value::Arguments(99)]),
    ]);
    let mut extras = FxHashMap::default();
    let mut mapped = FxHashMap::default();
    let mut fresh = FxHashMap::from_iter([
        (root, FxHashSet::from_iter(["arg".into()])),
        (dead_frame, FxHashSet::from_iter(["stale".into()])),
    ]);
    let mut captured = FxHashMap::from_iter([(root, FxHashMap::default())]);
    let compacted = compact_from_roots(
        &mut scopes,
        &[root],
        &[],
        &[],
        &[],
        &mut mapped,
        ArenaMut {
            objects: &mut objects,
            extras: &mut extras,
        },
        &mut fresh,
        &mut captured,
    );

    assert_eq!(compacted.indices, FxHashMap::from_iter([(root, root)]));
    assert_eq!(compacted.arguments, FxHashSet::from_iter([8]));
    assert!(scopes.get(root).is_some());
    assert!(scopes.get(dead_frame).is_none());
    assert!(scopes[root]["arg"] == Value::Arguments(8));
    assert_eq!(
        objects.keys().copied().collect::<FxHashSet<_>>(),
        FxHashSet::from_iter([8])
    );
    assert!(fresh.contains_key(&root));
    assert!(!fresh.contains_key(&dead_frame));
    assert!(captured.contains_key(&root));
}
