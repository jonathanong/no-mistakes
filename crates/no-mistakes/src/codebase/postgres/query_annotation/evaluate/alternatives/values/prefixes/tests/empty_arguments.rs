use super::{fixture, legacy, Value};
use crate::codebase::postgres::query_annotation::evaluate::alternatives::values::prefix_free_argument;
use crate::codebase::postgres::query_annotation::Function;
use crate::fx::FxHashSet;
use std::collections::BTreeMap;

fn scalar_case() -> fixture::Case {
    let mut case = fixture::load("arena_before");
    *case.objects.get_mut(&1).unwrap() = vec![
        Value::Prefix("unidentified".into(), true, None),
        Value::Primitive,
        Value::Function(
            std::sync::Arc::new(Function {
                start: 0,
                params: Vec::new(),
                body: Vec::new(),
                supported: true,
                asynchronous: false,
                arrow: true,
                self_name: None,
            }),
            std::path::PathBuf::from("callback.mts").into(),
            0,
        ),
    ]
    .into();
    case
}

fn parity(case: &fixture::Case, value: &Value) {
    assert!(prefix_free_argument(value, case.arena()));
    assert!(legacy::collect(value, case.arena(), &case.definite).is_empty());
    let mut changed = FxHashSet::default();
    super::super::super::changes(
        value,
        &Value::Prefix("after".into(), false, Some(7)),
        case.arena(),
        case.arena(),
        &mut changed,
        &case.definite,
        &mut None,
    );
    assert!(changed.is_empty());
}

#[test]
fn scalar_arguments_skip_without_full_edges_and_functions_do_not_supply_prefixes() {
    let case = scalar_case();
    let value = Value::Arguments(1);
    assert!(prefix_free_argument(&value, case.arena()));
    assert!(case.objects[&1].cached_references().is_none());
    let edges = case.objects[&1].references();
    assert_eq!(edges.environments, FxHashSet::from_iter([0]));
    for value in [
        value,
        Value::Array(1),
        Value::Promise(Box::new(Value::Arguments(1))),
        Value::Evaluated(Box::new(Value::Array(1)), false),
    ] {
        parity(&case, &value);
    }
    assert!(!prefix_free_argument(&Value::Unknown, case.arena()));
    assert!(!prefix_free_argument(&Value::Arguments(99), case.arena()));
}

#[test]
fn builder_and_nested_argument_edges_keep_the_original_walk() {
    let case = fixture::load("arena_before");
    assert!(!prefix_free_argument(&Value::Arguments(1), case.arena()));
    assert!(case.objects[&1].cached_references().is_none());
    case.objects[&1].references();
    assert!(!prefix_free_argument(&Value::Arguments(1), case.arena()));
    assert!(!legacy::collect(&Value::Arguments(1), case.arena(), &case.definite).is_empty());
    let mut case = scalar_case();
    case.objects.get_mut(&1).unwrap().push(Value::Array(2));
    case.objects.insert(2, vec![Value::Unknown]);
    case.objects[&1].references();
    assert!(!prefix_free_argument(&Value::Arguments(1), case.arena()));
}

#[test]
fn sparse_slots_are_live_and_dense_mutations_invalidate_only_the_current_snapshot() {
    let mut case = scalar_case();
    case.objects[&1].references();
    case.extras.insert(1, BTreeMap::new());
    parity(&case, &Value::Arguments(1));
    case.extras.get_mut(&1).unwrap().insert(3, Value::Unknown);
    assert!(!prefix_free_argument(&Value::Arguments(1), case.arena()));
    case.extras.clear();
    let snapshot = case.objects.clone();
    case.objects
        .get_mut(&1)
        .unwrap()
        .push(Value::Prefix("changed".into(), true, Some(9)));
    assert!(case.objects[&1].cached_references().is_none());
    assert!(!prefix_free_argument(&Value::Arguments(1), case.arena()));
    case.objects[&1].references();
    assert!(!prefix_free_argument(&Value::Arguments(1), case.arena()));
    let original = super::super::Arena {
        objects: &snapshot,
        extras: &case.extras,
    };
    assert!(prefix_free_argument(&Value::Arguments(1), original));
    case.objects.get_mut(&1).unwrap().pop();
    assert!(prefix_free_argument(&Value::Arguments(1), case.arena()));
    assert!(case.objects[&1].cached_references().is_none());
    case.objects[&1].references();
    case.definite.insert((1, 0));
    parity(&case, &Value::Arguments(1));
}

#[test]
fn dense_prefix_summary_is_shared_once_and_invalidated_by_snapshot_writes() {
    use crate::diagnostics::{with_observer, InvocationObserver};
    let mut case = scalar_case();
    let observer = InvocationObserver::new(true);
    with_observer(Some(observer.clone()), || {
        let snapshot = case.objects.clone();
        for _ in 0..16 {
            assert!(prefix_free_argument(&Value::Arguments(1), case.arena()));
            assert!(!snapshot[&1].contains_reference());
        }
        assert_eq!(
            observer.snapshot().work["postgres.annotation.dense_prefix_summary_builds"],
            1
        );
        assert!(case.objects[&1].cached_references().is_none());
        case.objects.get_mut(&1).unwrap().push(Value::Array(1));
        assert!(case.objects[&1].contains_reference());
        assert!(!snapshot[&1].contains_reference());
        assert_eq!(
            observer.snapshot().work["postgres.annotation.dense_prefix_summary_builds"],
            2
        );
        assert!(case.objects[&1].cached_references().is_none());
        case.objects.get_mut(&1).unwrap().pop();
        assert!(!case.objects[&1].contains_reference());
        assert_eq!(
            observer.snapshot().work["postgres.annotation.dense_prefix_summary_builds"],
            3
        );
        case.objects
            .get_mut(&1)
            .unwrap()
            .push(Value::Prefix("changed".into(), true, Some(9)));
        assert!(case.objects[&1].contains_reference());
        assert_eq!(
            observer.snapshot().work["postgres.annotation.dense_prefix_summary_builds"],
            4
        );
        assert!(case.objects[&1].cached_references().is_none());
    });
}

#[test]
fn dense_prefix_summary_disabled_observer_retains_no_work_ledger() {
    use crate::diagnostics::{with_observer, InvocationObserver};
    let case = scalar_case();
    let observer = InvocationObserver::new(false);
    with_observer(Some(observer.clone()), || {
        assert!(prefix_free_argument(&Value::Arguments(1), case.arena()));
        assert!(case.objects[&1].cached_references().is_none());
    });
    assert!(observer.snapshot().work.is_empty());
}
