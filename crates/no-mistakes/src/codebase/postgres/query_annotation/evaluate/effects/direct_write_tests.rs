use super::{
    tests::{collect_old, evaluator, same_state},
    Evaluator, Value,
};
use crate::fx::{FxHashMap, FxHashSet};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
type TestEvaluator<'a> = Evaluator<'a, fn(&str, &Path) -> Option<PathBuf>>;

fn prefix(id: u64) -> Value {
    Value::Prefix("SELECT 1".into(), false, Some(id))
}

fn legacy(evaluator: &mut TestEvaluator<'_>, roots: &[Value]) {
    evaluator.untrusted_array_intrinsics = true;
    let (ids, _) = collect_old(evaluator, roots);
    if ids.is_empty() {
        return;
    }
    evaluator.invalidate_mapped_freshness(&ids);
    evaluator.invalidated_builders.extend(ids.iter().copied());
    for scope in &mut evaluator.scopes {
        if !scope.contains_builders(&ids) {
            continue;
        }
        // Preserve the original no-op mutable access as the regression oracle.
        for value in scope.values_mut() {
            if matches!(value, Value::Prefix(_, _, Some(id)) if ids.contains(id)) {
                *value = Value::Unknown;
            }
        }
    }
}

#[test]
fn nested_only_invalidation_preserves_shared_scopes_and_warm_indexes_with_full_state_parity() {
    let shared = vec![prefix(7), Value::Arguments(1)].into();
    let scope = FxHashMap::from_iter([
        ("nested".into(), Value::Record(shared, vec![])),
        ("promise".into(), Value::Promise(Box::new(prefix(7)))),
        ("unrelated_direct".into(), prefix(99)),
    ]);
    let files = FxHashMap::default();
    let (mut old, mut new) = (evaluator(&files, scope.clone()), evaluator(&files, scope));
    for evaluator in [&mut old, &mut new] {
        evaluator
            .argument_objects
            .insert(1, vec![Value::Array(1), prefix(8), prefix(9)]);
        evaluator
            .argument_extra_slots
            .insert(1, BTreeMap::from_iter([(7, prefix(10))]));
        evaluator.definite_deleted_argument_slots.insert((1, 2));
        let names = FxHashSet::from_iter(["parameter".into()]);
        evaluator.fresh_mapped_parameters.insert(0, names.clone());
        evaluator
            .fresh_mapped_argument_bindings
            .insert(1, FxHashMap::from_iter([(0, names)]));
        evaluator.invalidated_builders.insert(100);
    }
    let old_snapshot = old.scopes[0].clone();
    let snapshot = new.scopes[0].clone();
    let references = new.scopes[0].references() as *const _;
    let selected = FxHashSet::from_iter([7]);
    assert!(new.scopes[0].contains_builders(&selected));
    let roots = [prefix(7), Value::Arguments(1)];
    for _ in 0..2 {
        legacy(&mut old, &roots);
        new.invalidate_opaque_builders(&roots);
        same_state(&old, &new);
        assert!(
            new.scopes[0].same_bindings(&snapshot),
            "no direct write means no COW or cache invalidation"
        );
        assert_eq!(
            new.scopes[0].cached_references().unwrap() as *const _,
            references
        );
    }
    assert!(
        !old.scopes[0].same_bindings(&old_snapshot),
        "the original path needlessly detaches this snapshot"
    );
    assert!(new.fresh_mapped_parameters.is_empty());
    assert!(new.fresh_mapped_argument_bindings.is_empty());
    assert!(new.untrusted_array_intrinsics);
    assert_eq!(
        new.invalidated_builders,
        FxHashSet::from_iter([1, 7, 8, 10, 100])
    );
    assert!(new.scopes[0]["unrelated_direct"] == prefix(99));
}

#[test]
fn matching_direct_binding_still_detaches_and_becomes_unknown() {
    let scope = FxHashMap::from_iter([
        ("direct".into(), prefix(7)),
        ("nested".into(), Value::Object(vec![prefix(7)].into())),
    ]);
    let files = FxHashMap::default();
    let (mut old, mut new) = (evaluator(&files, scope.clone()), evaluator(&files, scope));
    let snapshot = new.scopes[0].clone();
    new.scopes[0].references();
    legacy(&mut old, &[prefix(7)]);
    new.invalidate_opaque_builders(&[prefix(7)]);
    same_state(&old, &new);
    assert!(!new.scopes[0].same_bindings(&snapshot));
    assert!(new.scopes[0].cached_references().is_none());
    assert!(new.scopes[0]["direct"] == Value::Unknown);
    assert!(snapshot["direct"] == prefix(7));
}

#[test]
fn nonmatching_direct_binding_keeps_the_coarse_guard_and_shared_indexes() {
    let files = FxHashMap::default();
    let scope = FxHashMap::from_iter([("direct".into(), prefix(99))]);
    let (mut old, mut new) = (evaluator(&files, scope.clone()), evaluator(&files, scope));
    let snapshot = new.scopes[0].clone();
    let references = new.scopes[0].references() as *const _;
    assert!(!new.scopes[0].contains_builders(&FxHashSet::from_iter([7])));
    legacy(&mut old, &[prefix(7)]);
    new.invalidate_opaque_builders(&[prefix(7)]);
    same_state(&old, &new);
    assert!(new.scopes[0].same_bindings(&snapshot));
    assert_eq!(
        new.scopes[0].cached_references().unwrap() as *const _,
        references
    );
}
