use super::{binding_scan_count::Guard, join_with_sql, Scope, Value};
use crate::fx::FxHashMap;

fn scope() -> Scope {
    FxHashMap::from_iter([
        (
            "builder".into(),
            Value::Prefix("/* stable */ SELECT 1".into(), true, Some(7)),
        ),
        ("unknown".into(), Value::Unknown),
        ("argument".into(), Value::Arguments(9)),
    ])
    .into()
}

fn compare(before: &Scope, after: &Scope, initial: &Scope, whole_sql: bool) -> (Scope, usize) {
    let mut old = vec![before.clone()];
    let mut new = old.clone();
    let current = [after.clone()];
    let original = [initial.clone()];
    let mode = Guard::new(true);
    join_with_sql(&mut old, &current, &original, whole_sql);
    assert_eq!(mode.visits(), before.len(), "legacy visits every binding");
    drop(mode);
    let mode = Guard::new(false);
    join_with_sql(&mut new, &current, &original, whole_sql);
    assert!(old == new, "frozen legacy binding result");
    (new.pop().unwrap(), mode.visits())
}

#[test]
fn shared_snapshots_skip_actual_binding_visits_in_both_sql_modes() {
    for whole_sql in [false, true] {
        let shared = scope();
        let (result, visits) = compare(&shared, &shared, &shared, whole_sql);
        assert_eq!(visits, 0);
        assert!(result.same_bindings(&shared), "identity stays shared");
    }
}

#[test]
fn equal_but_distinct_maps_keep_the_legacy_path_in_both_sql_modes() {
    for whole_sql in [false, true] {
        let before = scope();
        let after = scope();
        assert!(!before.same_bindings(&after));
        let (result, visits) = compare(&before, &after, &before, whole_sql);
        assert_eq!(visits, before.len());
        assert!(result.same_bindings(&before));
    }
}

#[test]
fn shared_arms_still_check_a_different_initial_scope_and_its_missing_keys() {
    for whole_sql in [false, true] {
        let shared = scope();
        // The arms share identity; checking only those two would retain SQL proof.
        let mut initial = shared.clone();
        initial.remove("builder");
        let (result, visits) = compare(&shared, &shared, &initial, whole_sql);
        assert_eq!(visits, shared.len());
        assert!(result["builder"] == Value::Unknown);
        assert!(shared["builder"] != Value::Unknown);
        let distinct = scope();
        let (result, visits) = compare(&shared, &shared, &distinct, whole_sql);
        assert_eq!(visits, shared.len());
        assert!(
            result == shared,
            "equal initial maps still use legacy membership"
        );
    }
}

#[test]
fn copy_on_write_mutation_keeps_candidate_joins_and_original_snapshot() {
    for whole_sql in [false, true] {
        let before = scope();
        let mut after = before.clone();
        after.insert("argument".into(), Value::Arguments(11));
        after.remove("unknown");
        assert!(!before.same_bindings(&after));
        let (result, visits) = compare(&before, &after, &before, whole_sql);
        assert_eq!(visits, before.len());
        assert!(
            result["argument"]
                == Value::Joined(vec![Value::Arguments(9), Value::Arguments(11)].into())
        );
        assert!(result["unknown"] == Value::Unknown);
        assert!(before["argument"] == Value::Arguments(9));
        assert!(before.contains_key("unknown"));
    }
}
