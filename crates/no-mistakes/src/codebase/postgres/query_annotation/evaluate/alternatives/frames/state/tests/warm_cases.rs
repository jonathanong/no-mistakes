use super::*;
fn prime(state: &State) {
    for scope in &state.scopes {
        scope.references();
    }
    for slots in state.objects.values() {
        slots.references();
    }
}
#[test]
fn cold_pruning_keeps_global_dag_sharing_and_does_not_construct_summaries() {
    let state = fixture::saved();
    assert!(state
        .scopes
        .iter()
        .all(|scope| scope.cached_references().is_none()));
    assert!(state
        .objects
        .values()
        .all(|slots| slots.cached_references().is_none()));
    let cold = compare(&state);
    assert!(cold.legacy_visits > 0 && cold.bindings > 0 && cold.slots > 0);
    assert_eq!(
        cold.values, cold.legacy_visits,
        "cold fallback preserves global visited sharing"
    );
    assert_eq!(cold.summary_scans, 0);
    assert!(state
        .scopes
        .iter()
        .all(|scope| scope.cached_references().is_none()));
    assert!(state
        .objects
        .values()
        .all(|slots| slots.cached_references().is_none()));
    prime(&state);
    let warm = compare(&state);
    assert_eq!((warm.bindings, warm.slots), (0, 0));
    assert!(warm.values < warm.legacy_visits);
    eprintln!("prune work cold={cold:?}; explicitly primed warm={warm:?}");
}
#[test]
fn mutations_and_live_auxiliary_edges_keep_the_raw_oracle_state() {
    let mut state = fixture::saved();
    prime(&state);
    let original = state.clone();
    state.scopes[0].insert("changed".into(), Value::Arguments(9));
    assert!(state.scopes[0].cached_references().is_none());
    let changed = compare(&state);
    assert!(changed.bindings > 0);
    assert_eq!(changed.slots, 0);
    assert!(state.scopes[0].cached_references().is_none());
    assert_eq!(compare(&original).bindings, 0);
    state.objects.get_mut(&7).unwrap().push(Value::Arguments(9));
    assert!(state.objects[&7].cached_references().is_none());
    assert!(original.objects[&7].cached_references().is_some());
    assert!(compare(&state).slots > 0);
    assert!(state.objects[&7].cached_references().is_none());
    state.mapped.clear();
    state.captured.clear();
    state.extras.clear();
    compare(&state);
    state.objects.get_mut(&7).unwrap().clear();
    state.scopes[0].remove("changed");
    state.returned.clear();
    state.transient.clear();
    compare(&state);
}

#[test]
fn warmed_dense_conversion_preserves_shared_cache_and_moves_unique_slots() {
    let mut state = fixture::saved();
    prime(&state);
    let before = state.objects[&7].cached_references().unwrap() as *const _;
    let shared = state.objects[&7].clone();
    assert!(shared.shared());
    let values: crate::codebase::postgres::query_annotation::evaluate::value::Values =
        shared.into();
    assert_eq!(values.len(), 2);
    assert!(values[0] == Value::Arguments(7));
    assert_eq!(
        state.objects[&7].cached_references().unwrap() as *const _,
        before
    );
    let unique = state.objects.remove(&10).unwrap();
    assert!(!unique.shared());
    let pointer = unique.as_ptr();
    let values: crate::codebase::postgres::query_annotation::evaluate::value::Values =
        unique.into();
    assert_eq!(values.as_ptr(), pointer);
    assert!(values[0] == Value::Unknown);
}
