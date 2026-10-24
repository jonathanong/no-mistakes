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
        assert_eq!(visits, before.len(), "tiny scopes retain the borrowed scan");
        assert!(
            result["argument"]
                == Value::Joined(vec![Value::Arguments(9), Value::Arguments(11)].into())
        );
        assert!(result["unknown"] == Value::Unknown);
        assert!(before["argument"] == Value::Arguments(9));
        assert!(before.contains_key("unknown"));
    }
}

#[test]
fn dense_sparse_writes_compare_only_the_written_key_and_bulk_writes_fall_back() {
    for whole_sql in [false, true] {
        let mut before = scope();
        for index in 0..61 {
            before.insert(format!("stable{index}"), Value::Primitive);
        }
        let mut after = before.clone();
        after.insert("argument".into(), Value::Arguments(11));
        let (result, visits) = compare(&before, &after, &before, whole_sql);
        assert_eq!(visits, 1);
        assert!(result["argument"] != before["argument"]);
        assert!(result["builder"] == before["builder"]);
        let (_, visits) = compare(&after, &before, &before, whole_sql);
        assert_eq!(
            visits, 1,
            "the changed first arm also supplies a sparse key set"
        );
        // Bulk mutation has no proven key set; retain the borrowed legacy scan.
        after.entry("argument".into()).or_insert(Value::Unknown);
        let (_, visits) = compare(&before, &after, &before, whole_sql);
        assert_eq!(visits, before.len());
    }
}

#[test]
fn sparse_candidates_cannot_resurrect_a_binding_deleted_from_the_joined_arm() {
    for whole_sql in [false, true] {
        let mut initial = scope();
        for index in 0..61 {
            initial.insert(format!("stable{index}"), Value::Primitive);
        }
        let mut before = initial.clone();
        before.remove("argument");
        let mut after = initial.clone();
        after.insert("unknown".into(), Value::Arguments(11));
        let (result, visits) = compare(&before, &after, &initial, whole_sql);
        assert!(!result.contains_key("argument"));
        assert_eq!(
            visits, 1,
            "the absent dirty key is skipped before comparison"
        );
        assert!(
            result["unknown"] == Value::Joined(vec![Value::Unknown, Value::Arguments(11)].into())
        );
        assert!(result["builder"] == initial["builder"]);
        assert!(initial.contains_key("argument") && after.contains_key("argument"));
    }
}
